//! Valence-backed featured catalog integ (System actor + sqlite mem).

#![cfg(feature = "ssr")]
#![allow(missing_docs)]

use std::sync::Arc;

use uf_product::AppRegistration;
use uf_welcome::featured::{
    add, latest_change, list, list_changes, publish, record_change, remove, reorder, replace_all,
    restore, FeaturedError, PublishOutcome,
};
use valence::{
    register_noop_deletion_dispatcher_for_tests, router_key, Actor, InMemoryBackend, Valence,
    MEM_ENGINE_ID,
};

// Second known app so reorder can exercise two registry ids (welcome comes from the crate).
uf_product::inventory::submit! {
    AppRegistration {
        id: "apps",
        name: "Apps",
        description: "Fixture app for featured integ",
        icon: "Cube",
        route_path: "/apps",
        repository: None,
        crate_name: None,
        brand_seed: None,
        permission_manifest: None,
    }
}

fn system_valence() -> Valence {
    register_noop_deletion_dispatcher_for_tests();
    let key = router_key("default", MEM_ENGINE_ID);
    Valence::builder()
        .add_backend("default", Arc::new(InMemoryBackend::new()))
        .default_backend_key(key)
        .with_actor(Actor::System {
            operation: "featured_integ".into(),
        })
        .build()
        .expect("valence")
}

#[tokio::test]
async fn featured_add_list_orders_by_ordinal() {
    let v = system_valence();
    add(&v, "apps", 10).await.expect("add apps");
    add(&v, "welcome", 1).await.expect("add welcome");
    let rows = list(&v).await.expect("list");
    assert_eq!(
        rows.iter().map(|r| r.app_id.as_str()).collect::<Vec<_>>(),
        vec!["welcome", "apps"]
    );
    assert_eq!(rows[0].ordinal, 1);
    assert_eq!(rows[1].ordinal, 10);
}

#[tokio::test]
async fn featured_add_unknown_app() {
    let v = system_valence();
    let err = add(&v, "zz-unknown-app", 0).await.expect_err("unknown");
    assert!(matches!(err, FeaturedError::UnknownApp { .. }));
}

#[tokio::test]
async fn featured_add_duplicate() {
    let v = system_valence();
    add(&v, "welcome", 0).await.expect("add");
    let err = add(&v, "welcome", 1).await.expect_err("dup");
    assert!(matches!(err, FeaturedError::Duplicate { .. }));
}

#[tokio::test]
async fn featured_remove_ok() {
    let v = system_valence();
    add(&v, "welcome", 0).await.expect("add");
    remove(&v, "welcome").await.expect("remove");
    assert!(list(&v).await.expect("list").is_empty());
}

#[tokio::test]
async fn featured_remove_missing() {
    let v = system_valence();
    let err = remove(&v, "welcome").await.expect_err("missing");
    assert!(matches!(err, FeaturedError::NotFound { .. }));
}

#[tokio::test]
async fn featured_reorder_persists_order() {
    let v = system_valence();
    add(&v, "welcome", 0).await.expect("add welcome");
    add(&v, "apps", 1).await.expect("add apps");
    reorder(&v, &["apps".to_string(), "welcome".to_string()])
        .await
        .expect("reorder");
    let rows = list(&v).await.expect("list");
    assert_eq!(rows[0].app_id, "apps");
    assert_eq!(rows[0].ordinal, 0);
    assert_eq!(rows[1].app_id, "welcome");
    assert_eq!(rows[1].ordinal, 1);
}

#[tokio::test]
async fn featured_reorder_missing_err() {
    let v = system_valence();
    add(&v, "welcome", 0).await.expect("add");
    let err = reorder(&v, &["welcome".to_string(), "apps".to_string()])
        .await
        .expect_err("missing apps");
    assert!(matches!(err, FeaturedError::NotFound { .. }));
}

#[tokio::test]
async fn featured_authenticated_without_admin_cud_denied() {
    register_noop_deletion_dispatcher_for_tests();
    let key = router_key("default", MEM_ENGINE_ID);
    let v = Valence::builder()
        .add_backend("default", Arc::new(InMemoryBackend::new()))
        .default_backend_key(key)
        .with_actor(Actor::User {
            user_id: "alice".into(),
        })
        .build()
        .expect("valence");
    let err = add(&v, "welcome", 0)
        .await
        .expect_err("AUTHENTICATED without WelcomeAdmin must fail CUD");
    assert!(
        matches!(err, FeaturedError::Service { .. }),
        "expected privacy/service denial, got {err:?}"
    );
}

fn alice_valence() -> Valence {
    register_noop_deletion_dispatcher_for_tests();
    let key = router_key("default", MEM_ENGINE_ID);
    Valence::builder()
        .add_backend("default", Arc::new(InMemoryBackend::new()))
        .default_backend_key(key)
        .with_actor(Actor::User {
            user_id: "alice".into(),
        })
        .build()
        .expect("valence")
}

fn ids(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| (*s).to_string()).collect()
}

async fn featured_ids(v: &Valence) -> Vec<String> {
    list(v)
        .await
        .expect("list")
        .into_iter()
        .map(|r| r.app_id)
        .collect()
}

#[tokio::test]
async fn featured_replace_all_adds_removes_and_orders_happy() {
    let v = system_valence();
    add(&v, "welcome", 0).await.expect("add welcome");
    replace_all(&v, &ids(&["apps", "welcome"]))
        .await
        .expect("replace");
    assert_eq!(featured_ids(&v).await, ids(&["apps", "welcome"]));
    replace_all(&v, &ids(&["welcome"])).await.expect("replace");
    assert_eq!(featured_ids(&v).await, ids(&["welcome"]));
    replace_all(&v, &[]).await.expect("replace empty");
    assert!(featured_ids(&v).await.is_empty());
}

#[tokio::test]
async fn featured_replace_all_rejects_repeat_and_unknown_sad() {
    let v = system_valence();
    let err = replace_all(&v, &ids(&["apps", "apps"]))
        .await
        .expect_err("repeat");
    assert!(matches!(err, FeaturedError::Duplicate { .. }));
    let err = replace_all(&v, &ids(&["zz-unknown-app"]))
        .await
        .expect_err("unknown");
    assert!(matches!(err, FeaturedError::UnknownApp { .. }));
    assert!(featured_ids(&v).await.is_empty());
}

#[tokio::test]
async fn featured_publish_records_change_and_moves_version_happy() {
    let v = system_valence();
    assert!(latest_change(&v).await.expect("latest").is_none());

    let PublishOutcome::Published(first) = publish(&v, None, &ids(&["welcome"]), None)
        .await
        .expect("publish")
    else {
        panic!("expected Published");
    };
    assert_eq!(first.before, Vec::<String>::new());
    assert_eq!(first.after, ids(&["welcome"]));
    assert_eq!(first.summary, "Added Welcome");
    assert_eq!(featured_ids(&v).await, ids(&["welcome"]));

    let PublishOutcome::Published(second) =
        publish(&v, None, &ids(&["welcome", "apps"]), Some(&first.id))
            .await
            .expect("publish")
    else {
        panic!("expected Published");
    };
    assert_eq!(second.summary, "Added Apps at the end");
    let latest = latest_change(&v).await.expect("latest").expect("some");
    assert_eq!(latest.id, second.id);
    let history = list_changes(&v).await.expect("history");
    assert_eq!(
        history.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(),
        vec![second.id.as_str(), first.id.as_str()]
    );
}

#[tokio::test]
async fn featured_publish_stale_version_conflicts_sad() {
    let v = system_valence();
    let PublishOutcome::Published(first) = publish(&v, None, &ids(&["welcome"]), None)
        .await
        .expect("publish")
    else {
        panic!("expected Published");
    };
    // A draft started before `first` was published.
    let outcome = publish(&v, None, &ids(&["apps"]), None)
        .await
        .expect("publish");
    assert_eq!(outcome, PublishOutcome::Conflict(Some(first)));
    assert_eq!(featured_ids(&v).await, ids(&["welcome"]));
}

#[tokio::test]
async fn featured_publish_same_list_is_unchanged() {
    let v = system_valence();
    let outcome = publish(&v, None, &[], None).await.expect("publish");
    assert_eq!(outcome, PublishOutcome::Unchanged);
    assert!(list_changes(&v).await.expect("history").is_empty());
}

#[tokio::test]
async fn featured_restore_puts_back_list_before_change_happy() {
    let v = system_valence();
    let PublishOutcome::Published(first) = publish(&v, None, &ids(&["welcome"]), None)
        .await
        .expect("publish")
    else {
        panic!("expected Published");
    };
    let PublishOutcome::Published(second) =
        publish(&v, None, &ids(&["apps", "welcome"]), Some(&first.id))
            .await
            .expect("publish")
    else {
        panic!("expected Published");
    };
    let restored = restore(&v, None, &second.id).await.expect("restore");
    assert_eq!(featured_ids(&v).await, ids(&["welcome"]));
    assert_eq!(restored.before, ids(&["apps", "welcome"]));
    assert_eq!(restored.after, ids(&["welcome"]));
    assert!(restored.summary.starts_with("Restored an earlier list"));
    // Undo of the restore brings the second list back.
    restore(&v, None, &restored.id).await.expect("undo restore");
    assert_eq!(featured_ids(&v).await, ids(&["apps", "welcome"]));
}

#[tokio::test]
async fn featured_restore_unknown_change_sad() {
    let v = system_valence();
    let err = restore(&v, None, "missing").await.expect_err("missing");
    assert!(matches!(err, FeaturedError::NotFound { .. }));
}

#[cfg(feature = "e2e-seed")]
#[tokio::test]
async fn featured_clear_history_happy() {
    use uf_welcome::featured::clear_history;

    let v = system_valence();
    publish(&v, None, &ids(&["welcome"]), None)
        .await
        .expect("publish");
    clear_history(&v).await.expect("clear");
    assert!(list_changes(&v).await.expect("history").is_empty());
}

#[cfg(feature = "e2e-seed")]
#[tokio::test]
async fn featured_clear_history_refuses_non_system_sad() {
    use uf_welcome::featured::clear_history;

    let err = clear_history(&alice_valence())
        .await
        .expect_err("only System may clear history");
    assert!(matches!(err, FeaturedError::Service { .. }));
}

#[tokio::test]
async fn featured_history_without_admin_denied() {
    let v = alice_valence();
    let err = record_change(
        &v,
        None,
        "Added Welcome".into(),
        Vec::new(),
        ids(&["welcome"]),
    )
    .await
    .expect_err("AUTHENTICATED without WelcomeAdmin must not write history");
    assert!(
        matches!(err, FeaturedError::Service { .. }),
        "expected privacy/service denial, got {err:?}"
    );
}
