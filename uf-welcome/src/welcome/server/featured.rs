//! Featured-app catalog server functions.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// App shortcut DTO shared by welcome usage and featured cards.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppLinkDto {
    /// Registered app id when known.
    pub app_id: String,
    /// Display label.
    pub label: String,
    /// Navigation href.
    pub link: String,
}

/// One selectable app for the featured admin picker.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManageableAppDto {
    /// [`uf_product::AppRegistration::id`].
    pub app_id: String,
    /// Display name.
    pub name: String,
    /// Route prefix.
    pub route_path: String,
}

/// One app as the featured admin page shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeaturedAdminAppDto {
    /// [`uf_product::AppRegistration::id`].
    pub app_id: String,
    /// Display name.
    pub name: String,
    /// One-line description from the registration.
    pub description: String,
}

/// One published change to the featured list, ready to show.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeaturedChangeDto {
    /// Change id (pass to [`restore_featured`]).
    pub id: String,
    /// Display name of who published it ("Someone" when unknown).
    pub actor_name: String,
    /// What changed, in words.
    pub summary: String,
    /// RFC 3339 instant for `<time datetime>`.
    pub created_at: String,
    /// Short label such as "Nov 18, 10:04 PM UTC".
    pub created_label: String,
}

/// Everything the featured admin page needs on load.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeaturedAdminDto {
    /// Published list, in order.
    pub published: Vec<FeaturedAdminAppDto>,
    /// Every registered app, sorted by name (the Add picker filters out featured ones).
    pub catalog: Vec<FeaturedAdminAppDto>,
    /// Version of the published list: the newest change id, `None` before any change.
    pub version: Option<String>,
    /// The newest change, for "Last published by".
    pub last_change: Option<FeaturedChangeDto>,
}

/// Outcome of [`publish_featured`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PublishFeaturedResult {
    /// Published; `version` is the new version and `change_id` the change to undo.
    Published {
        /// New version of the published list.
        version: String,
        /// Change recorded for this publish.
        change_id: String,
    },
    /// The draft already matched the published list.
    Unchanged,
    /// Someone published since the draft started.
    Conflict {
        /// Who published the newer change.
        by_name: String,
    },
}

/// Tower-session key e2e hosts set when Gauge/`WelcomeAdmin` is unavailable.
#[cfg(feature = "ssr")]
pub const E2E_WELCOME_ADMIN_SESSION_KEY: &str = "uf_e2e_welcome_admin";

#[cfg(feature = "ssr")]
const WELCOME_ADMIN: &str = "WelcomeAdmin";

#[cfg(feature = "ssr")]
fn map_featured_error(e: crate::welcome::featured::FeaturedError) -> ServerFnError {
    ServerFnError::new(e.to_string())
}

#[cfg(feature = "ssr")]
fn rows_to_dtos(rows: Vec<crate::welcome::featured::FeaturedAppRow>) -> Vec<AppLinkDto> {
    use uf_product::AppRegistry;

    let registry = AppRegistry::auto_discover();
    rows.into_iter()
        .filter_map(|row| {
            registry
                .iter()
                .find(|r| r.id == row.app_id)
                .map(|reg| AppLinkDto {
                    app_id: row.app_id,
                    label: reg.name.to_string(),
                    link: reg.route_path.to_string(),
                })
        })
        .collect()
}

#[cfg(feature = "ssr")]
async fn e2e_welcome_admin_from_session() -> bool {
    use leptos_axum::extract;
    use tower_sessions::Session;

    let Ok(session) = extract::<Session>().await else {
        return false;
    };
    session
        .get::<bool>(E2E_WELCOME_ADMIN_SESSION_KEY)
        .await
        .ok()
        .flatten()
        .unwrap_or(false)
}

#[cfg(feature = "ssr")]
fn registration_dto(app_id: &str) -> Option<FeaturedAdminAppDto> {
    uf_product::AppRegistry::auto_discover()
        .iter()
        .find(|r| r.id == app_id)
        .map(|r| FeaturedAdminAppDto {
            app_id: r.id.to_string(),
            name: r.name.to_string(),
            description: r.description.to_string(),
        })
}

/// The signed-in user as a Valence record, recorded as who made a change.
#[cfg(feature = "ssr")]
async fn session_actor() -> Option<valence::RecordId> {
    let ctx = higgs::Higgs::from_request().await.ok()?;
    let id = ctx.session_user_id()?;
    uf_product::ssr::session_user_record_id(id).ok()
}

/// Current profile display name for `actor`, or "Someone".
#[cfg(feature = "ssr")]
async fn actor_display_name(
    valence: &valence::Valence,
    actor: Option<&valence::RecordId>,
) -> String {
    use lepton_identity::generated::UserProfile;
    use valence::{Model, RecordPredicate};

    const UNKNOWN: &str = "Someone";
    let Some(actor) = actor else {
        return UNKNOWN.to_string();
    };
    let profile = UserProfile::query(valence, valence::use_!(r"In **featured history**, we **read the curator's public display name** from their profile so Welcome admins see who published each change, without storing the name in the history entry."))
        .where_user(RecordPredicate::Equals(actor.clone()))
        .first()
        .await;
    match profile {
        Ok(Some(p)) if !p.display_name().trim().is_empty() => p.display_name().trim().to_string(),
        Ok(_) => UNKNOWN.to_string(),
        Err(err) => {
            leptos::logging::warn!("featured history: profile lookup failed for {actor}: {err}");
            UNKNOWN.to_string()
        }
    }
}

#[cfg(feature = "ssr")]
async fn change_dto(
    valence: &valence::Valence,
    change: &crate::welcome::featured::FeaturedChangeRow,
) -> FeaturedChangeDto {
    let mut dto = change_dto_without_name(change);
    dto.actor_name = actor_display_name(valence, change.actor.as_ref()).await;
    dto
}

/// Record a change made by the single-row calls. The row write already succeeded,
/// so a history failure is logged rather than reported as a failed edit.
#[cfg(feature = "ssr")]
async fn record_legacy_change(valence: &valence::Valence, before: Vec<String>) {
    let actor = session_actor().await;
    if let Err(err) = crate::welcome::featured::record_since(valence, actor, before).await {
        leptos::logging::warn!("featured: recording history failed: {err}");
    }
}

#[cfg(feature = "ssr")]
fn harness_valence() -> Option<std::sync::Arc<valence::Valence>> {
    use_context::<std::sync::Arc<valence::Valence>>()
}

#[cfg(feature = "ssr")]
fn harness_session_valence() -> Option<valence::Valence> {
    // Prefer the harness Valence as-is (User/Anonymous). Do not elevate for reads —
    // catalog rows should be seeded under an AUTHENTICATED actor.
    harness_valence().map(|v| (*v).clone())
}

#[cfg(feature = "ssr")]
async fn require_welcome_admin() -> Result<WelcomeAdminAccess, ServerFnError> {
    #[cfg(feature = "admin-permissions")]
    {
        uf_product::permissions::require_permission(WELCOME_ADMIN).await?;
        let ctx = higgs::Higgs::from_request().await?;
        Ok(WelcomeAdminAccess::Higgs(ctx))
    }
    #[cfg(not(feature = "admin-permissions"))]
    {
        if e2e_welcome_admin_from_session().await {
            return Ok(WelcomeAdminAccess::E2eHarness);
        }
        Err(ServerFnError::new(
            higgs::server_runtime::permission_denied_payload(WELCOME_ADMIN),
        ))
    }
}

#[cfg(feature = "ssr")]
enum WelcomeAdminAccess {
    #[cfg(feature = "admin-permissions")]
    Higgs(higgs::Higgs),
    #[cfg(not(feature = "admin-permissions"))]
    E2eHarness,
}

#[cfg(feature = "ssr")]
impl WelcomeAdminAccess {
    /// Session Valence for featured CUD (WelcomeAdmin enforced by Valence policy).
    fn session_valence(&self) -> Result<valence::Valence, ServerFnError> {
        match self {
            #[cfg(feature = "admin-permissions")]
            Self::Higgs(ctx) => ctx
                .valence()
                .map_err(|e| ServerFnError::new(format!("Failed to build session Valence: {e}"))),
            #[cfg(not(feature = "admin-permissions"))]
            Self::E2eHarness => harness_session_valence()
                .ok_or_else(|| ServerFnError::new("e2e featured Valence context missing")),
        }
    }
}

/// Whether the signed-in user may manage featured apps (`WelcomeAdmin`).
///
/// # Errors
///
/// Does not return [`ServerFnError`] for missing session, Higgs, Valence, or
/// permission-check failure; those paths return `Ok(false)`. Transport /
/// extractor failures may still surface as [`ServerFnError`].
#[uf_product_macros::server]
pub async fn can_manage_welcome_featured() -> Result<bool, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        #[cfg(feature = "admin-permissions")]
        {
            let Ok(ctx) = higgs::Higgs::from_request().await else {
                return Ok(false);
            };
            if ctx.session_user_id().is_none() {
                return Ok(false);
            }
            return Ok(uf_product::permissions::has_permission(WELCOME_ADMIN).await);
        }
        #[cfg(not(feature = "admin-permissions"))]
        {
            return Ok(e2e_welcome_admin_from_session().await);
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(false)
    }
}

/// Featured apps for the welcome page (authenticated Valence read).
///
/// # Errors
///
/// Does not return [`ServerFnError`] for anonymous viewers, missing Valence, or
/// catalog list failure; those paths degrade to `Ok([])`. Transport / extractor
/// failures may still surface as [`ServerFnError`].
#[uf_product_macros::server]
pub async fn get_featured_apps() -> Result<Vec<AppLinkDto>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        // Harness Valence in context (e2e host) wins over a Higgs shell with no catalog.
        let harness_ready = uf_product::telemetry::usage::resolve_usage_viewer_key()
            .await
            .is_some()
            || e2e_welcome_admin_from_session().await;
        if harness_ready {
            if let Some(valence) = harness_session_valence() {
                if let Ok(rows) = crate::welcome::featured::list(&valence).await {
                    return Ok(rows_to_dtos(rows));
                }
            }
        }

        if let Ok(ctx) = higgs::Higgs::from_request().await {
            if ctx.session_user_id().is_none() {
                return Ok(Vec::new());
            }
            let Ok(valence) = ctx.valence() else {
                return Ok(Vec::new());
            };
            let Ok(rows) = crate::welcome::featured::list(&valence).await else {
                return Ok(Vec::new());
            };
            return Ok(rows_to_dtos(rows));
        }

        Ok(Vec::new())
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(Vec::new())
    }
}

/// Apps available to feature (WelcomeAdmin).
///
/// # Errors
///
/// Returns [`ServerFnError`] when the caller lacks `WelcomeAdmin` (permission
/// denied payload), Higgs/Valence setup fails under `admin-permissions`, or the
/// server-fn transport fails. Without the `ssr` feature this returns `Ok([])`.
#[uf_product_macros::server]
pub async fn list_manageable_apps() -> Result<Vec<ManageableAppDto>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let _access = require_welcome_admin().await?;
        use uf_product::AppRegistry;
        let mut apps: Vec<ManageableAppDto> = AppRegistry::auto_discover()
            .iter()
            .map(|r| ManageableAppDto {
                app_id: r.id.to_string(),
                name: r.name.to_string(),
                route_path: r.route_path.to_string(),
            })
            .collect();
        apps.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(apps)
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(Vec::new())
    }
}

/// Add an app to the featured catalog (WelcomeAdmin).
///
/// # Errors
///
/// Returns [`ServerFnError`] when the caller lacks `WelcomeAdmin`, session Valence
/// cannot be built, [`crate::welcome::featured::add`] fails (mapped from
/// [`crate::welcome::featured::FeaturedError`]: unknown app, duplicate, or
/// Valence service failure), the new row's `app_id` is missing from
/// [`uf_product::AppRegistry`], or the call is made without the `ssr` feature
/// (`"ssr only"`).
#[uf_product_macros::server]
pub async fn add_featured_app(app_id: String, ordinal: i64) -> Result<AppLinkDto, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use uf_product::AppRegistry;

        let access = require_welcome_admin().await?;
        let valence = access.session_valence()?;
        let before = crate::welcome::featured::current_app_ids(&valence)
            .await
            .map_err(map_featured_error)?;
        let row = crate::welcome::featured::add(&valence, &app_id, ordinal)
            .await
            .map_err(map_featured_error)?;
        record_legacy_change(&valence, before).await;
        let registry = AppRegistry::auto_discover();
        let reg = registry
            .iter()
            .find(|r| r.id == row.app_id)
            .ok_or_else(|| ServerFnError::new(format!("unknown app_id: {}", row.app_id)))?;
        Ok(AppLinkDto {
            app_id: row.app_id,
            label: reg.name.to_string(),
            link: reg.route_path.to_string(),
        })
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = (app_id, ordinal);
        Err(ServerFnError::new("ssr only"))
    }
}

/// Remove a featured app by app id or record id (WelcomeAdmin).
///
/// # Errors
///
/// Returns [`ServerFnError`] when the caller lacks `WelcomeAdmin`, session Valence
/// cannot be built, [`crate::welcome::featured::remove`] fails (mapped from
/// [`crate::welcome::featured::FeaturedError`]: not found or Valence service
/// failure), or the call is made without the `ssr` feature (`"ssr only"`).
#[uf_product_macros::server]
pub async fn remove_featured_app(app_id_or_id: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let access = require_welcome_admin().await?;
        let valence = access.session_valence()?;
        let before = crate::welcome::featured::current_app_ids(&valence)
            .await
            .map_err(map_featured_error)?;
        crate::welcome::featured::remove(&valence, &app_id_or_id)
            .await
            .map_err(map_featured_error)?;
        record_legacy_change(&valence, before).await;
        Ok(())
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = app_id_or_id;
        Err(ServerFnError::new("ssr only"))
    }
}

/// Reorder featured apps by app_id list (WelcomeAdmin).
///
/// # Errors
///
/// Returns [`ServerFnError`] when the caller lacks `WelcomeAdmin`, session Valence
/// cannot be built, [`crate::welcome::featured::reorder`] fails (mapped from
/// [`crate::welcome::featured::FeaturedError`]: not found or Valence service
/// failure), or the call is made without the `ssr` feature (`"ssr only"`).
#[uf_product_macros::server]
pub async fn reorder_featured_apps(app_ids: Vec<String>) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let access = require_welcome_admin().await?;
        let valence = access.session_valence()?;
        let before = crate::welcome::featured::current_app_ids(&valence)
            .await
            .map_err(map_featured_error)?;
        crate::welcome::featured::reorder(&valence, &app_ids)
            .await
            .map_err(map_featured_error)?;
        record_legacy_change(&valence, before).await;
        Ok(())
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = app_ids;
        Err(ServerFnError::new("ssr only"))
    }
}

/// Published list, catalog, version and last change for the admin page (WelcomeAdmin).
///
/// # Errors
///
/// Returns [`ServerFnError`] when the caller lacks `WelcomeAdmin` (permission
/// denied payload), session Valence cannot be built, the catalog or history read
/// fails, or the call is made without the `ssr` feature (`"ssr only"`).
#[uf_product_macros::server]
pub async fn get_featured_admin() -> Result<FeaturedAdminDto, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use uf_product::AppRegistry;

        let access = require_welcome_admin().await?;
        let valence = access.session_valence()?;
        let published = crate::welcome::featured::current_app_ids(&valence)
            .await
            .map_err(map_featured_error)?
            .iter()
            .filter_map(|id| registration_dto(id))
            .collect();
        let mut catalog: Vec<FeaturedAdminAppDto> = AppRegistry::auto_discover()
            .iter()
            .filter_map(|r| registration_dto(r.id))
            .collect();
        catalog.sort_by(|a, b| a.name.cmp(&b.name));
        let latest = crate::welcome::featured::latest_change(&valence)
            .await
            .map_err(map_featured_error)?;
        let last_change = match &latest {
            Some(change) => Some(change_dto(&valence, change).await),
            None => None,
        };
        Ok(FeaturedAdminDto {
            published,
            catalog,
            version: latest.map(|c| c.id),
            last_change,
        })
    }
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("ssr only"))
    }
}

/// Publish the whole ordered featured list (WelcomeAdmin).
///
/// `based_on_version` is the version the draft started from. When someone
/// published since, nothing is written and [`PublishFeaturedResult::Conflict`]
/// names them.
///
/// # Errors
///
/// Returns [`ServerFnError`] when the caller lacks `WelcomeAdmin`, session Valence
/// cannot be built, the list names an unknown or repeated app, the write or
/// history record fails, or the call is made without the `ssr` feature.
#[uf_product_macros::server]
pub async fn publish_featured(
    app_ids: Vec<String>,
    based_on_version: Option<String>,
) -> Result<PublishFeaturedResult, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::welcome::featured::{publish, PublishOutcome};

        let access = require_welcome_admin().await?;
        let valence = access.session_valence()?;
        let actor = session_actor().await;
        let outcome = publish(&valence, actor, &app_ids, based_on_version.as_deref())
            .await
            .map_err(map_featured_error)?;
        Ok(match outcome {
            PublishOutcome::Published(change) => PublishFeaturedResult::Published {
                version: change.id.clone(),
                change_id: change.id,
            },
            PublishOutcome::Unchanged => PublishFeaturedResult::Unchanged,
            PublishOutcome::Conflict(latest) => PublishFeaturedResult::Conflict {
                by_name: actor_display_name(
                    &valence,
                    latest.as_ref().and_then(|c| c.actor.as_ref()),
                )
                .await,
            },
        })
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = (app_ids, based_on_version);
        Err(ServerFnError::new("ssr only"))
    }
}

/// Every published change to the featured list, newest first (WelcomeAdmin).
///
/// # Errors
///
/// Returns [`ServerFnError`] when the caller lacks `WelcomeAdmin`, session Valence
/// cannot be built, the history read fails, or the call is made without `ssr`.
#[uf_product_macros::server]
pub async fn list_featured_history() -> Result<Vec<FeaturedChangeDto>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use std::collections::HashMap;

        let access = require_welcome_admin().await?;
        let valence = access.session_valence()?;
        let changes = crate::welcome::featured::list_changes(&valence)
            .await
            .map_err(map_featured_error)?;
        let mut names: HashMap<Option<String>, String> = HashMap::new();
        let mut out = Vec::with_capacity(changes.len());
        for change in &changes {
            let key = change.actor.as_ref().map(ToString::to_string);
            let name = match names.get(&key) {
                Some(name) => name.clone(),
                None => {
                    let name = actor_display_name(&valence, change.actor.as_ref()).await;
                    names.insert(key, name.clone());
                    name
                }
            };
            let mut dto = change_dto_without_name(change);
            dto.actor_name = name;
            out.push(dto);
        }
        Ok(out)
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(Vec::new())
    }
}

#[cfg(feature = "ssr")]
fn change_dto_without_name(
    change: &crate::welcome::featured::FeaturedChangeRow,
) -> FeaturedChangeDto {
    FeaturedChangeDto {
        id: change.id.clone(),
        actor_name: String::new(),
        summary: change.summary.clone(),
        created_at: change.created_at.to_rfc3339(),
        created_label: change
            .created_at
            .format("%b %-d, %-I:%M %p UTC")
            .to_string(),
    }
}

/// Put the featured list back to how it was before change `change_id` (WelcomeAdmin).
///
/// Records the restore as a new change and returns it, so the page can undo the
/// restore too.
///
/// # Errors
///
/// Returns [`ServerFnError`] when the caller lacks `WelcomeAdmin`, session Valence
/// cannot be built, the change id is unknown, the write fails, or the call is
/// made without the `ssr` feature.
#[uf_product_macros::server]
pub async fn restore_featured(change_id: String) -> Result<FeaturedChangeDto, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let access = require_welcome_admin().await?;
        let valence = access.session_valence()?;
        let actor = session_actor().await;
        let change = crate::welcome::featured::restore(&valence, actor, &change_id)
            .await
            .map_err(map_featured_error)?;
        Ok(change_dto(&valence, &change).await)
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = change_id;
        Err(ServerFnError::new("ssr only"))
    }
}
