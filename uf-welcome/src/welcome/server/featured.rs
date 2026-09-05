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
        let row = crate::welcome::featured::add(&valence, &app_id, ordinal)
            .await
            .map_err(map_featured_error)?;
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
        crate::welcome::featured::remove(&valence, &app_id_or_id)
            .await
            .map_err(map_featured_error)
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
        crate::welcome::featured::reorder(&valence, &app_ids)
            .await
            .map_err(map_featured_error)
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = app_ids;
        Err(ServerFnError::new("ssr only"))
    }
}
