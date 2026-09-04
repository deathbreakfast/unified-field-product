//! Harness-only seed endpoint for Playwright.

use axum::http::StatusCode;
use axum::Json;
use chrono::{TimeZone, Utc};
use serde::Deserialize;
use serde_json::json;
use spectra::try_log_event_at;
use tower_sessions::Session;
use uf_notifications_core::{send_notification, Notification, SendNotification};
use uf_product::telemetry::usage::{E2E_USAGE_VIEWER_SESSION_KEY, PAGE_VIEW_LOG_TABLE};
use uf_product::workspace_search::{SearchDocumentDraft, SearchDocumentWriter};
use valence::{Actor, Model, RecordId, RecordPredicate};

use crate::e2e_permissions::{E2E_PERMISSION_ALLOW_FLAG, E2E_PERMISSION_ALLOW_SESSION_KEY};
use crate::e2e_spectra::e2e_spectra;
use crate::e2e_valence::{e2e_system_valence, e2e_valence, store_minted_ids, take_minted_ids};
use crate::gate_demos::{write_e2e_auth_kind, E2eAuthKind, E2E_VERIFIED_SESSION_USER};
use std::sync::atomic::Ordering;
use uf_welcome::E2E_WELCOME_ADMIN_SESSION_KEY;

#[derive(Debug, Deserialize)]
pub struct SeedPageView {
    /// [`uf_product::AppRegistration::id`].
    pub app_id: String,
    /// Spectra `viewer_key` (defaults to e2e usage viewer).
    #[serde(default)]
    pub viewer_key: Option<String>,
    /// Seconds before now (for ordering).
    #[serde(default)]
    pub age_secs: i64,
}

#[derive(Debug, Deserialize)]
pub struct SeedNotification {
    #[serde(default = "default_kind")]
    pub kind: String,
    pub title: String,
    #[serde(default = "default_message")]
    pub message: String,
    #[serde(default)]
    pub url: Option<String>,
}

fn default_kind() -> String {
    "e2e".into()
}

fn default_message() -> String {
    "Seeded by uf-product-ui-e2e".into()
}

#[derive(Debug, Deserialize)]
pub struct SeedRequest {
    /// E2e auth kind: `anonymous`, `authenticated_verified`, `authenticated_unverified`.
    #[serde(default = "default_auth")]
    pub auth: String,
    /// Override Spectra / welcome usage viewer key (defaults from auth kind).
    #[serde(default)]
    pub usage_viewer: Option<String>,
    /// When true, set e2e welcome-admin session key for featured mutations.
    #[serde(default)]
    pub welcome_admin: bool,
    /// When true, allow `e2e.permission.allow` via harness PermissionBackend.
    #[serde(default)]
    pub permission_allow: bool,
    /// Optional page-view events into mem Spectra (usage cards).
    #[serde(default)]
    pub page_views: Vec<SeedPageView>,
    /// Notifications to mint with System Valence (never session create).
    #[serde(default)]
    pub notifications: Vec<SeedNotification>,
    /// When true, keep previously minted rows and append new ones (Photon push probes).
    #[serde(default)]
    pub append: bool,
}

fn default_auth() -> String {
    E2eAuthKind::Anonymous.as_str().to_string()
}

fn default_viewer_for(kind: E2eAuthKind) -> Option<&'static str> {
    match kind {
        E2eAuthKind::AuthenticatedVerified => Some("e2e-user"),
        E2eAuthKind::AuthenticatedUnverified => Some("e2e-unverified"),
        E2eAuthKind::Anonymous => None,
    }
}

fn recipient_record_id() -> RecordId {
    let (table, id) = E2E_VERIFIED_SESSION_USER
        .split_once(':')
        .expect("e2e session user is table:id");
    RecordId::new(table, id)
}

fn recipient_user_id_str() -> String {
    E2E_VERIFIED_SESSION_USER
        .split_once(':')
        .map(|(_, id)| id.to_string())
        .unwrap_or_else(|| E2E_VERIFIED_SESSION_USER.to_string())
}

fn bare_notification_id(row: &Notification) -> Option<String> {
    row.id().map(|thing| {
        let s = thing.to_string();
        s.split(':')
            .next_back()
            .unwrap_or(&s)
            .trim_matches(|c| c == '⟨' || c == '⟩')
            .to_string()
    })
}

const NOTIFICATION_TABLE: &str = "notification";

/// Physically remove a notification row (Model::delete only marks pending_deletion under
/// the noop dispatcher and can leave ownership / recreate issues across seeds).
async fn hard_delete_notification_id(system: &valence::Valence, id: &str) -> bool {
    let candidates = [
        id.to_string(),
        id.trim_start_matches("notification:")
            .trim_matches(|c| c == '⟨' || c == '⟩')
            .to_string(),
    ];
    let mut last_err = None;
    for bare in candidates {
        if bare.is_empty() {
            continue;
        }
        match system.backend_for_table(NOTIFICATION_TABLE) {
            Ok(backend) => match backend.delete_record(NOTIFICATION_TABLE, &bare).await {
                Ok(()) => {
                    valence::read_cache::invalidate(NOTIFICATION_TABLE, &bare);
                    let _ = valence::ownership::OwnershipService::mark_deleted_ownership(
                        NOTIFICATION_TABLE,
                        &bare,
                        system,
                    )
                    .await;
                    log::info!("e2e seed: hard-deleted notification {bare}");
                    return true;
                }
                Err(valence::Error::NotFound(_)) => {
                    valence::read_cache::invalidate(NOTIFICATION_TABLE, &bare);
                    let _ = valence::ownership::OwnershipService::mark_deleted_ownership(
                        NOTIFICATION_TABLE,
                        &bare,
                        system,
                    )
                    .await;
                }
                Err(err) => {
                    last_err = Some(err);
                }
            },
            Err(err) => {
                last_err = Some(err);
            }
        }
    }
    if let Some(err) = last_err {
        log::warn!("e2e seed: hard-delete notification {id} failed: {err}");
    } else {
        log::debug!("e2e seed: notification {id} already absent");
    }
    false
}

/// Hard-delete every notification the e2e user can still read, plus tracked mint ids.
async fn wipe_recipient_notifications(system: &valence::Valence, recipient: RecordId) -> usize {
    let tracked = take_minted_ids();
    let owner = e2e_system_valence().with_actor(Actor::User {
        user_id: recipient_user_id_str(),
    });
    let mut deleted = 0usize;
    let mut seen = std::collections::HashSet::<String>::new();

    for id in tracked {
        if seen.insert(id.clone()) && hard_delete_notification_id(system, &id).await {
            deleted = deleted.saturating_add(1);
        }
    }

    for round in 0..40 {
        let batch = match Notification::query(&owner)
            .where_user(RecordPredicate::Equals(recipient.clone()))
            .limit(100)
            .await
        {
            Ok(rows) => rows,
            Err(err) => {
                log::warn!("e2e seed: wipe query failed (round {round}): {err}");
                break;
            }
        };
        if batch.is_empty() {
            log::info!("e2e seed: wipe complete after {round} rounds, deleted={deleted}");
            break;
        }
        let before = deleted;
        for row in &batch {
            if let Some(id) = bare_notification_id(row) {
                if seen.insert(id.clone()) && hard_delete_notification_id(system, &id).await {
                    deleted = deleted.saturating_add(1);
                }
            }
        }
        if deleted == before {
            log::warn!(
                "e2e seed: wipe made no progress with {} visible rows remaining (round {round})",
                batch.len()
            );
            break;
        }
        log::info!(
            "e2e seed: wipe round {round} deleted_total={deleted} batch={}",
            batch.len()
        );
    }

    deleted
}

pub async fn seed_data(
    session: Session,
    Json(body): Json<SeedRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let kind = E2eAuthKind::parse(&body.auth);
    write_e2e_auth_kind(&session, kind).await.map_err(|err| {
        log::error!("e2e seed: write auth kind failed: {err}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let viewer = body
        .usage_viewer
        .clone()
        .or_else(|| default_viewer_for(kind).map(str::to_string));

    if let Some(ref viewer) = viewer {
        session
            .insert(E2E_USAGE_VIEWER_SESSION_KEY, viewer.clone())
            .await
            .map_err(|err| {
                log::error!("e2e seed: insert usage viewer failed: {err}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
    } else {
        session
            .remove::<String>(E2E_USAGE_VIEWER_SESSION_KEY)
            .await
            .map_err(|err| {
                log::error!("e2e seed: remove usage viewer failed: {err}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
    }

    if body.welcome_admin {
        session
            .insert(E2E_WELCOME_ADMIN_SESSION_KEY, true)
            .await
            .map_err(|err| {
                log::error!("e2e seed: insert welcome_admin failed: {err}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
    } else {
        session
            .remove::<bool>(E2E_WELCOME_ADMIN_SESSION_KEY)
            .await
            .map_err(|err| {
                log::error!("e2e seed: remove welcome_admin failed: {err}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
    }

    if body.permission_allow {
        E2E_PERMISSION_ALLOW_FLAG.store(true, Ordering::SeqCst);
        session
            .insert(E2E_PERMISSION_ALLOW_SESSION_KEY, true)
            .await
            .map_err(|err| {
                log::error!("e2e seed: insert permission_allow failed: {err}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
    } else {
        E2E_PERMISSION_ALLOW_FLAG.store(false, Ordering::SeqCst);
        session
            .remove::<bool>(E2E_PERMISSION_ALLOW_SESSION_KEY)
            .await
            .map_err(|err| {
                log::error!("e2e seed: remove permission_allow failed: {err}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
    }

    // Process-global Valence: clear featured catalog so scenarios do not leak rows.
    {
        use uf_welcome::featured::clear_all;
        let valence = e2e_valence().with_actor(Actor::System {
            operation: "e2e_seed_clear_featured".into(),
        });
        if let Err(err) = clear_all(&valence).await {
            log::warn!("e2e seed: clear_all featured failed: {err}");
        }
    }

    // Seed IndexedDemoItem under the verified e2e user (SE writes workspace index).
    let mut workspace_search_seeded = false;
    if matches!(kind, E2eAuthKind::AuthenticatedVerified) {
        use uf_product::generated::IndexedDemoItem;
        use uf_product::workspace_search::demo::{DEMO_APP_ID, DEMO_KIND, DEMO_SOURCE_TABLE};

        let valence = e2e_valence().with_actor(Actor::System {
            operation: "e2e_seed_workspace_search".into(),
        });
        let owner = RecordId::new("user", "e2e-user");
        let row = IndexedDemoItem::new(
            owner.clone(),
            "WorkspaceBeaconAlpha".into(),
            "/workspace-search-hit".into(),
        )
        .map_err(|err| {
            log::error!("e2e seed: IndexedDemoItem::new failed: {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        IndexedDemoItem::upsert("e2e-ws-1", row, &valence)
            .await
            .map_err(|err| {
                log::error!("e2e seed: IndexedDemoItem upsert failed: {err}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        // Belt-and-suspenders: SE may no-op / fail without rolling back the source row.
        SearchDocumentWriter::upsert(
            &valence,
            SearchDocumentDraft {
                user: owner,
                app_id: DEMO_APP_ID.into(),
                source_table: DEMO_SOURCE_TABLE.into(),
                source_id: "e2e-ws-1".into(),
                title: "WorkspaceBeaconAlpha".into(),
                searchable_text: "WorkspaceBeaconAlpha".into(),
                link: "/workspace-search-hit".into(),
                kind: DEMO_KIND.into(),
            },
        )
        .await
        .map_err(|err| {
            log::error!("e2e seed: SearchDocumentWriter upsert failed: {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        workspace_search_seeded = true;
    }

    // Ensure global Spectra is installed before try_log_event_at.
    let spectra = e2e_spectra();
    let now = Utc::now().timestamp();
    for pv in &body.page_views {
        let event_viewer = pv
            .viewer_key
            .clone()
            .or_else(|| viewer.clone())
            .unwrap_or_else(|| "anonymous".into());
        let ts = Utc
            .timestamp_opt(now - pv.age_secs.max(0), 0)
            .single()
            .ok_or(StatusCode::BAD_REQUEST)?;
        try_log_event_at(
            PAGE_VIEW_LOG_TABLE,
            &json!({
                "path": format!("/{}", pv.app_id),
                "app_id": pv.app_id,
                "app_name": pv.app_id,
                "route_prefix": format!("/{}", pv.app_id),
                "surface": "main_shell",
                "auth": "authenticated",
                "email_verified": "unknown",
                "viewer_key": event_viewer,
                "nav_kind": "e2e_seed",
                "referrer_path": "",
                "outcome": "ok",
                "permission_name": "",
                "role_count": 0,
            }),
            ts,
        );
    }
    if !body.page_views.is_empty() {
        spectra.flush_persist().await.map_err(|err| {
            log::error!("e2e seed: spectra flush_persist failed: {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    }

    let mut recent_preview = Vec::<String>::new();
    if let Some(ref viewer) = viewer {
        use uf_product::telemetry::usage::{recent_apps_for_viewer, UsageQueryOptions};
        if let Ok(apps) =
            recent_apps_for_viewer(spectra.as_ref(), viewer, &UsageQueryOptions::default()).await
        {
            recent_preview = apps.into_iter().map(|a| a.app_id).collect();
        }
    }

    let system = e2e_system_valence().with_actor(Actor::System {
        operation: "e2e_seed_send_notification".into(),
    });
    let recipient = recipient_record_id();
    // Wipe unless appending (Photon live-push). Empty `notifications` clears the bell.
    let wiped = if body.append {
        0usize
    } else {
        wipe_recipient_notifications(&system, recipient.clone()).await
    };

    let mut minted = Vec::with_capacity(body.notifications.len());
    for row in &body.notifications {
        let dto = send_notification(
            SendNotification {
                user_id: recipient.clone(),
                kind: row.kind.clone(),
                title: row.title.clone(),
                message: row.message.clone(),
                url: row.url.clone(),
                data_json: None,
            },
            &system,
        )
        .await
        .map_err(|err| {
            log::error!("e2e seed: send_notification failed: {err}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        minted.push(dto.notification_id.to_string());
    }
    if body.append {
        store_minted_ids(minted.clone());
    } else {
        let _ = take_minted_ids();
        store_minted_ids(minted.clone());
    }

    Ok(Json(serde_json::json!({
        "ok": true,
        "auth": kind.as_str(),
        "usage_viewer": viewer,
        "welcome_admin": body.welcome_admin,
        "permission_allow": body.permission_allow,
        "page_views": body.page_views.len(),
        "recent_preview": recent_preview,
        "workspace_search_seeded": workspace_search_seeded,
        "minted": minted.len(),
        "wiped": wiped,
        "notification_ids": minted,
        "append": body.append,
    })))
}
