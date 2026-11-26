//! Owner-only Welcome data: pinned apps, settings and Waiting on you.

#![cfg(feature = "ssr")]
#![allow(missing_docs)]

use std::sync::Arc;

use uf_product::AppRegistration;
use uf_welcome::welcome::server::{
    list_pinned, load_settings, pin, reorder_pinned, save_start_here_hidden, unpin,
    waiting_for_user, PinnedError, WAITING_LIMIT,
};
use valence::{
    register_noop_deletion_dispatcher_for_tests, router_key, Actor, InMemoryBackend, RecordId,
    Valence, MEM_ENGINE_ID,
};

// Extra registered apps so pins can hold more than the crate's own `welcome`.
uf_product::inventory::submit! {
    AppRegistration {
        id: "apps",
        name: "Apps",
        description: "Fixture app for pinned integ",
        icon: "Cube",
        route_path: "/apps",
        repository: None,
        crate_name: None,
        brand_seed: None,
        permission_manifest: None,
    }
}

uf_product::inventory::submit! {
    AppRegistration {
        id: "notes",
        name: "Notes",
        description: "Fixture app for pinned integ",
        icon: "Note",
        route_path: "/notes",
        repository: None,
        crate_name: None,
        brand_seed: Some("#225588"),
        permission_manifest: None,
    }
}

fn as_user(user: &str) -> Actor {
    Actor::User {
        user_id: user.into(),
    }
}

fn user(id: &str) -> RecordId {
    RecordId::new("user", id)
}

/// Valence reads this once, on the first ownership check in the process. The
/// sqlite backend behind the notification store has no unified ownership
/// fetch, so every fixture turns it off before anything reads it (as the
/// uf-notifications suites do).
fn disable_unified_ownership_fetch() {
    if std::env::var_os("VALENCE_OWNERSHIP_UNIFIED_FETCH").is_none() {
        std::env::set_var("VALENCE_OWNERSHIP_UNIFIED_FETCH", "0");
    }
}

fn mem_valence(owner: &str) -> Valence {
    disable_unified_ownership_fetch();
    register_noop_deletion_dispatcher_for_tests();
    Valence::builder()
        .add_backend("default", Arc::new(InMemoryBackend::new()))
        .default_backend_key(router_key("default", MEM_ENGINE_ID))
        .with_actor(as_user(owner))
        .build()
        .expect("valence")
}

#[tokio::test]
async fn pin_reorder_and_unpin_happy() {
    let v = mem_valence("alice");
    let alice = user("alice");
    pin(&v, &alice, "welcome").await.expect("pin welcome");
    pin(&v, &alice, "apps").await.expect("pin apps");
    pin(&v, &alice, "notes").await.expect("pin notes");
    assert_eq!(
        list_pinned(&v, &alice).await.expect("list"),
        ["welcome", "apps", "notes"]
    );

    reorder_pinned(
        &v,
        &alice,
        &["notes".into(), "welcome".into(), "apps".into()],
    )
    .await
    .expect("reorder");
    assert_eq!(
        list_pinned(&v, &alice).await.expect("list"),
        ["notes", "welcome", "apps"]
    );

    unpin(&v, &alice, "welcome").await.expect("unpin");
    assert_eq!(
        list_pinned(&v, &alice).await.expect("list"),
        ["notes", "apps"]
    );

    // Unpinned apps can be pinned again, at the end.
    pin(&v, &alice, "welcome").await.expect("re-pin");
    assert_eq!(
        list_pinned(&v, &alice).await.expect("list"),
        ["notes", "apps", "welcome"]
    );
}

#[tokio::test]
async fn pin_rejects_unknown_and_duplicate_sad() {
    let v = mem_valence("alice");
    let alice = user("alice");
    assert_eq!(
        pin(&v, &alice, "zz-unknown").await,
        Err(PinnedError::UnknownApp("zz-unknown".into()))
    );
    pin(&v, &alice, "apps").await.expect("pin");
    assert_eq!(
        pin(&v, &alice, "apps").await,
        Err(PinnedError::Duplicate("apps".into()))
    );
    assert_eq!(
        unpin(&v, &alice, "notes").await,
        Err(PinnedError::NotFound("notes".into()))
    );
    assert_eq!(
        reorder_pinned(&v, &alice, &["notes".into()]).await,
        Err(PinnedError::NotFound("notes".into()))
    );
}

#[tokio::test]
async fn another_user_cannot_read_or_change_pins_sad() {
    let v = mem_valence("alice");
    let alice = user("alice");
    pin(&v, &alice, "apps").await.expect("pin");

    let bob = v.with_actor(as_user("bob"));
    let peek = list_pinned(&bob, &alice).await;
    assert!(
        peek.as_ref().map_or(true, Vec::is_empty),
        "bob must not see alice's pins: {peek:?}"
    );
    assert!(unpin(&bob, &alice, "apps").await.is_err());
    assert!(pin(&bob, &alice, "notes").await.is_err());
    assert_eq!(list_pinned(&v, &alice).await.expect("list"), ["apps"]);
}

#[tokio::test]
async fn settings_default_then_saved_happy() {
    let v = mem_valence("alice");
    let alice = user("alice");
    assert!(
        !load_settings(&v, &alice)
            .await
            .expect("load")
            .start_here_hidden
    );

    save_start_here_hidden(&v, &alice, true)
        .await
        .expect("hide");
    assert!(
        load_settings(&v, &alice)
            .await
            .expect("load")
            .start_here_hidden
    );

    save_start_here_hidden(&v, &alice, false)
        .await
        .expect("show");
    assert!(
        !load_settings(&v, &alice)
            .await
            .expect("load")
            .start_here_hidden
    );
}

#[tokio::test]
async fn another_user_cannot_read_settings_sad() {
    let v = mem_valence("alice");
    let alice = user("alice");
    save_start_here_hidden(&v, &alice, true)
        .await
        .expect("hide");

    let bob = v.with_actor(as_user("bob"));
    let peek = load_settings(&bob, &alice).await;
    assert!(
        peek.as_ref().map_or(true, |s| !s.start_here_hidden),
        "bob must not read alice's settings: {peek:?}"
    );
}

mod waiting {
    use super::*;

    use chrono::Utc;
    use lepton_identity::generated::{User, UserStatus, UserUserType};
    use uf_notifications_core::{send_notification, SendNotification};
    use valence::{
        register_backend_logical_names, DatabaseBackend, DatabaseRouter, Model,
        RegisterBackendLogicalNamesOptions, SqliteBackend, SQLITE_ENGINE_ID,
    };

    uf_notifications_core::inventory::submit! {
        uf_notifications_core::NotificationKind {
            id: "welcometest.approve",
            app: "welcometest",
            label: "Request to approve",
            description: "Someone asked you to approve access.",
            defaults: uf_notifications_core::Channels::IN_APP_ONLY,
            asks_action: true,
            urgent: false,
        }
    }

    uf_notifications_core::inventory::submit! {
        uf_notifications_core::NotificationKind {
            id: "welcometest.answered",
            app: "welcometest",
            label: "Request answered",
            description: "Your access request was answered.",
            defaults: uf_notifications_core::Channels::IN_APP_ONLY,
            asks_action: false,
            urgent: false,
        }
    }

    const OWNER: &str = "welcome-waiting-owner";
    const PEER: &str = "welcome-waiting-peer";

    /// **Test:** sqlite-mem Valence as System for fixture setup only.
    async fn system_valence() -> Valence {
        disable_unified_ownership_fetch();
        register_noop_deletion_dispatcher_for_tests();
        let backend: Arc<dyn DatabaseBackend> = Arc::new(
            SqliteBackend::connect_memory()
                .await
                .expect("SqliteBackend::connect_memory"),
        );
        let mut router = DatabaseRouter::new();
        register_backend_logical_names(
            &mut router,
            backend,
            &["default"],
            RegisterBackendLogicalNamesOptions::default(),
        );
        let v = Valence::builder()
            .database_router(Arc::new(router))
            .default_backend_key(router_key("default", SQLITE_ENGINE_ID))
            .with_actor(Actor::System {
                operation: "welcome_waiting_test".into(),
            })
            .build()
            .expect("valence");
        for id in [OWNER, PEER] {
            let now = Utc::now();
            let row = User::new(
                Some(UserUserType::Person),
                Some("test-password-hash".into()),
                Some(UserStatus::Active),
                None,
                None,
                Some(now),
                None,
                None,
                now,
                now,
            )
            .expect("build user");
            User::upsert(id, row, &v, valence::use_!(r"**Test:** Fixture **User** save for the Welcome Waiting on you suite, so notifications have recipients. CI and developers running the suite only."))
                .await
                .expect("seed user");
        }
        v
    }

    async fn send(
        system: &Valence,
        to: &str,
        kind: &str,
        title: &str,
    ) -> uf_notifications_core::NotificationDto {
        send_notification(
            SendNotification {
                user_id: user(to),
                kind: kind.into(),
                title: title.into(),
                message: format!("body for {title}"),
                url: Some("/notifications".into()),
                data_json: None,
                subject: None,
            },
            system,
        )
        .await
        .expect("send")
    }

    #[tokio::test]
    async fn waiting_returns_only_unread_asks_happy() {
        let system = system_valence().await;
        let read_me = send(&system, OWNER, "welcometest.approve", "approve old").await;
        send(&system, OWNER, "welcometest.approve", "approve new").await;
        send(&system, OWNER, "welcometest.answered", "answered").await;
        send(&system, OWNER, "unregistered.kind", "unregistered").await;
        send(&system, PEER, "welcometest.approve", "peer approve").await;

        let owner = system.with_actor(as_user(OWNER));
        uf_notifications_api::mark_read_for_user(&owner, read_me.notification_id)
            .await
            .expect("mark read");

        let waiting = waiting_for_user(&owner, user(OWNER))
            .await
            .expect("waiting");
        let titles: Vec<&str> = waiting.iter().map(|n| n.title.as_str()).collect();
        assert_eq!(titles, ["approve new"]);
        assert!(waiting.iter().all(|n| n.asks_action && !n.is_read));
    }

    #[tokio::test]
    async fn waiting_caps_at_limit() {
        let system = system_valence().await;
        for i in 0..(WAITING_LIMIT + 2) {
            send(&system, OWNER, "welcometest.approve", &format!("ask {i}")).await;
        }
        let owner = system.with_actor(as_user(OWNER));
        let waiting = waiting_for_user(&owner, user(OWNER))
            .await
            .expect("waiting");
        assert_eq!(waiting.len(), WAITING_LIMIT as usize);
    }
}
