//! Usage card server functions: recent pages (Valence) and recent / popular
//! apps (Spectra).

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use super::featured::AppLinkDto;

/// The last page the viewer opened in one app.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecentPageDto {
    /// The app the page belongs to.
    pub app: AppLinkDto,
    /// Page title (last breadcrumb item, or the app name).
    pub title: String,
    /// Pathname to reopen the page.
    pub route: String,
    /// When the page was opened (RFC 3339, UTC).
    pub viewed_at: String,
}

#[cfg(feature = "ssr")]
fn map_usage(links: Vec<uf_product::telemetry::usage::UsageAppLink>) -> Vec<AppLinkDto> {
    links
        .into_iter()
        .map(|a| {
            super::featured::app_link_for_id(&a.app_id).unwrap_or(AppLinkDto {
                app_id: a.app_id,
                label: a.label,
                link: a.link,
                icon: String::new(),
                description: String::new(),
                brand_seed: None,
                can_open: true,
            })
        })
        .collect()
}

/// Map a usage query result for the page: a failed query is an error, never
/// an empty list.
#[cfg(feature = "ssr")]
fn map_usage_result(
    result: Result<
        Vec<uf_product::telemetry::usage::UsageAppLink>,
        uf_product::telemetry::usage::UsageQueryError,
    >,
) -> Result<Vec<AppLinkDto>, ServerFnError> {
    result
        .map(map_usage)
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[cfg(feature = "ssr")]
fn usage_router() -> Result<std::sync::Arc<spectra::SpectraRouter>, ServerFnError> {
    use std::sync::Arc;

    use spectra::{Spectra, SpectraRouter};
    use uf_product::telemetry::usage::UsageQueryError;

    if let Some(spectra) = use_context::<Arc<Spectra>>() {
        return Ok(spectra.router());
    }
    SpectraRouter::try_global()
        .ok_or_else(|| ServerFnError::new(UsageQueryError::SpectraUnavailable.to_string()))
}

/// Per-user recent apps from Spectra page-view telemetry.
///
/// Returns an empty list when nobody is signed in.
///
/// # Errors
///
/// Returns [`ServerFnError`] when the host has no Spectra handle or the
/// Spectra query fails, so the page shows an error instead of an empty card.
#[uf_product_macros::server]
pub async fn get_recent_apps() -> Result<Vec<AppLinkDto>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use uf_product::telemetry::usage::{
            recent_apps_for_viewer_on, resolve_usage_viewer_key, UsageQueryOptions,
        };

        let Some(uid) = resolve_usage_viewer_key().await else {
            return Ok(Vec::new());
        };
        let router = usage_router()?;
        map_usage_result(
            recent_apps_for_viewer_on(router.as_ref(), &uid, &UsageQueryOptions::default()).await,
        )
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(Vec::new())
    }
}

/// Fleet-wide popular apps from Spectra page-view telemetry.
///
/// Aggregate only: the list never carries per-person counts. A signed-in (or
/// e2e) viewer is still required; without one the call returns an empty list.
///
/// # Errors
///
/// Same as [`get_recent_apps`]: a missing Spectra handle or a failed query is
/// a [`ServerFnError`].
#[uf_product_macros::server]
pub async fn get_popular_apps() -> Result<Vec<AppLinkDto>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use uf_product::telemetry::usage::{
            popular_apps_on, resolve_usage_viewer_key, UsageQueryOptions,
        };

        // Require a signed-in / e2e viewer even though popular is fleet-wide.
        if resolve_usage_viewer_key().await.is_none() {
            return Ok(Vec::new());
        }
        let router = usage_router()?;
        map_usage_result(popular_apps_on(router.as_ref(), &UsageQueryOptions::default()).await)
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(Vec::new())
    }
}

/// The last page the viewer opened in each app: newest first, at most 8,
/// from the last 30 days.
///
/// Returns an empty list when nobody is signed in. Pages of apps this host no
/// longer registers are left out.
///
/// # Errors
///
/// Returns [`ServerFnError`] when the request context or session Valence
/// cannot be built, or the recent-pages query fails.
#[uf_product_macros::server]
pub async fn get_recent_pages() -> Result<Vec<RecentPageDto>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use uf_product::telemetry::recent_pages::{list_recent_pages, RECENT_PAGES_LIMIT};

        let Some((valence, user)) = super::owner_session().await? else {
            return Ok(Vec::new());
        };
        let rows = list_recent_pages(
            &valence,
            &user,
            RECENT_PAGES_LIMIT,
            chrono::Duration::days(30),
        )
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
        Ok(rows
            .into_iter()
            .filter_map(|row| {
                super::featured::app_link_for_id(&row.app_id).map(|app| RecentPageDto {
                    app,
                    title: row.title,
                    route: row.route,
                    viewed_at: row.viewed_at.to_rfc3339(),
                })
            })
            .collect())
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(Vec::new())
    }
}

/// Delete the caller's recent pages, of any age, as the session actor.
///
/// # Errors
///
/// Returns [`ServerFnError`] when nobody is signed in, the session Valence
/// cannot be built, or a delete fails.
#[uf_product_macros::server]
pub async fn clear_my_history() -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let (valence, user) = super::require_owner_session().await?;
        uf_product::telemetry::recent_pages::clear_recent_pages(&valence, &user)
            .await
            .map(|_| ())
            .map_err(|e| ServerFnError::new(e.to_string()))
    }
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new("ssr only"))
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;
    use uf_product::telemetry::usage::{UsageAppLink, UsageQueryError};

    #[test]
    fn failed_usage_query_is_an_error_sad() {
        let err = map_usage_result(Err(UsageQueryError::QueryFailed {
            cause: "backend timeout".into(),
        }))
        .expect_err("query failure must not become an empty list");
        assert!(err.to_string().contains("backend timeout"));
    }

    #[test]
    fn usage_links_keep_unregistered_apps_with_fallbacks() {
        let got = map_usage_result(Ok(vec![UsageAppLink {
            app_id: "not-registered-xyz".into(),
            label: "Fallback".into(),
            link: "/fallback".into(),
        }]))
        .expect("ok");
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].label, "Fallback");
        assert!(got[0].icon.is_empty());
        assert!(got[0].can_open);
    }
}
