//! Usage card server functions (Spectra-backed).

use leptos::prelude::*;

use super::featured::AppLinkDto;

#[cfg(feature = "ssr")]
fn map_usage(links: Vec<uf_product::telemetry::usage::UsageAppLink>) -> Vec<AppLinkDto> {
    links
        .into_iter()
        .map(|a| AppLinkDto {
            app_id: a.app_id,
            label: a.label,
            link: a.link,
        })
        .collect()
}

#[cfg(feature = "ssr")]
fn usage_router() -> Option<std::sync::Arc<spectra::SpectraRouter>> {
    use std::sync::Arc;

    use spectra::{Spectra, SpectraRouter};

    if let Some(spectra) = use_context::<Arc<Spectra>>() {
        return Some(spectra.router());
    }
    SpectraRouter::try_global()
}

/// Per-user recent apps from Spectra page-view telemetry.
///
/// Returns an empty list when the viewer or Spectra handle is unavailable
/// (degrade for hosts without Higgs / Spectra).
///
/// # Errors
///
/// Does not return [`ServerFnError`] for missing viewer, missing Spectra, or
/// query failure; those paths degrade to `Ok([])`. Transport / extractor failures
/// from the Leptos server-fn layer may still surface as [`ServerFnError`].
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
        let Some(router) = usage_router() else {
            return Ok(Vec::new());
        };
        match recent_apps_for_viewer_on(router.as_ref(), &uid, &UsageQueryOptions::default()).await
        {
            Ok(apps) => Ok(map_usage(apps)),
            Err(_) => Ok(Vec::new()),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(Vec::new())
    }
}

/// Per-user most-used apps from Spectra page-view telemetry.
///
/// # Errors
///
/// Same degrade-to-`Ok([])` contract as [`get_recent_apps`]: missing viewer,
/// missing Spectra, or query failure do not become [`ServerFnError`]. Transport
/// failures may still surface as [`ServerFnError`].
#[uf_product_macros::server]
pub async fn get_my_most_used() -> Result<Vec<AppLinkDto>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use uf_product::telemetry::usage::{
            most_used_for_viewer_on, resolve_usage_viewer_key, UsageQueryOptions,
        };

        let Some(uid) = resolve_usage_viewer_key().await else {
            return Ok(Vec::new());
        };
        let Some(router) = usage_router() else {
            return Ok(Vec::new());
        };
        match most_used_for_viewer_on(router.as_ref(), &uid, &UsageQueryOptions::default()).await {
            Ok(apps) => Ok(map_usage(apps)),
            Err(_) => Ok(Vec::new()),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(Vec::new())
    }
}

/// Fleet-wide popular apps from Spectra page-view telemetry.
///
/// # Errors
///
/// Same degrade-to-`Ok([])` contract as [`get_recent_apps`]. A signed-in (or e2e)
/// viewer is still required before the fleet query runs; without one the call
/// returns `Ok([])`. Transport failures may still surface as [`ServerFnError`].
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
        let Some(router) = usage_router() else {
            return Ok(Vec::new());
        };
        match popular_apps_on(router.as_ref(), &UsageQueryOptions::default()).await {
            Ok(apps) => Ok(map_usage(apps)),
            Err(_) => Ok(Vec::new()),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(Vec::new())
    }
}
