//! Server functions and pure helpers for the apps directory.
//!
//! Data comes from the in-memory [`uf_product::AppRegistry`] (public registration
//! metadata only — no Valence reads, no permission gate). [`crate::server::get_apps_page`] and
//! [`crate::server::get_app_overview`] are the endpoints [`crate::AppsIndexPage`] and
//! [`crate::AppDetailPage`] call.
//!
//! ## Search / filter
//!
//! [`crate::server::filter_apps_by_query`] mirrors the index search box. [`crate::server::page_apps`] applies
//! the orbital over-fetch slice. Unit tests in this module cover happy and sad
//! paths for sort, filter, lookup, and pagination.
//!
//! ## Failure modes
//!
//! | Function | `Ok` shapes | `Err(ServerFnError)` |
//! |----------|-------------|----------------------|
//! | [`crate::server::get_apps_page`] | [`orbital_paging::Page<AppDirectoryItem>`] (possibly empty) | SSR / transport failure |
//! | [`crate::server::get_app_overview`] | `Some(AppOverview)` or `None` for unknown slug | SSR / transport failure |
//! | [`crate::server::get_apps`] (legacy) | `Vec<AppDirectoryItem>` | SSR / transport failure |
//!
//! Unknown slugs and empty registries are not errors. [`crate::server::find_app_overview`] returns
//! `None` when the slug is missing.

use leptos::prelude::*;
use orbital_paging::Page;
use serde::{Deserialize, Serialize};

/// Page size used by the apps index infinite scroll.
pub const APPS_PAGE_SIZE: u32 = 12;

/// One entry in the apps directory listing, derived from an app's registration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppDirectoryItem {
    /// Human-friendly application name.
    pub name: String,
    /// App id, used as the URL slug on the detail page.
    pub slug: String,
    /// Short app description shown in the directory grid.
    pub description: String,
    /// The app's primary route prefix (e.g. `"/counter"`).
    pub route_path: String,
    /// Optional source repository URL.
    pub repository: Option<String>,
    /// Optional crates.io package name.
    pub crate_name: Option<String>,
}

/// Detail-page summary for a single registered app.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppOverview {
    /// Human-friendly application name.
    pub name: String,
    /// Short app description.
    pub description: String,
    /// The app's primary route prefix (e.g. `"/counter"`).
    pub route_path: String,
    /// Optional source repository URL.
    pub repository: Option<String>,
    /// Optional crates.io package name (docs link to docs.rs when set).
    pub crate_name: Option<String>,
}

/// Sort apps lexicographically by display name (stable directory order).
pub fn sort_apps_by_name(apps: &mut [AppDirectoryItem]) {
    apps.sort_by(|a, b| a.name.cmp(&b.name));
}

/// Case-insensitive name/description filter used by [`crate::server::get_apps_page`].
///
/// Blank / whitespace-only queries leave `apps` unchanged.
pub fn filter_apps_by_query(apps: &mut Vec<AppDirectoryItem>, query: Option<&str>) {
    let Some(q) = query else {
        return;
    };
    let q_lower = q.trim().to_lowercase();
    if q_lower.is_empty() {
        return;
    }
    apps.retain(|a| {
        a.name.to_lowercase().contains(&q_lower) || a.description.to_lowercase().contains(&q_lower)
    });
}

/// Look up a directory item by slug and map it to [`AppOverview`].
pub fn find_app_overview(apps: &[AppDirectoryItem], slug: &str) -> Option<AppOverview> {
    apps.iter().find(|a| a.slug == slug).map(|a| AppOverview {
        name: a.name.clone(),
        description: a.description.clone(),
        route_path: a.route_path.clone(),
        repository: a.repository.clone(),
        crate_name: a.crate_name.clone(),
    })
}

/// Paginate a sorted/filtered apps list with the orbital over-fetch pattern.
pub fn page_apps(apps: Vec<AppDirectoryItem>, offset: u32, limit: u32) -> Page<AppDirectoryItem> {
    let total_count: Option<u64> = if offset == 0 {
        Some(apps.len() as u64)
    } else {
        None
    };

    let sliced: Vec<AppDirectoryItem> = apps
        .into_iter()
        .skip(offset as usize)
        .take(limit.saturating_add(1) as usize)
        .collect();

    Page::from_oversized(sliced, limit, total_count)
}

#[cfg(feature = "ssr")]
async fn maybe_simulate_delay() {
    use std::time::Duration;
    let ms: u64 = std::env::var("UF_SIMULATE_APPS_DELAY_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);

    if ms > 0 {
        tokio::time::sleep(Duration::from_millis(ms)).await;
    }
}

#[cfg(not(feature = "ssr"))]
#[allow(dead_code)]
const fn maybe_simulate_delay() {}

/// Snapshot every Orbital app registered via `uf_app!`/`uf_app!` as directory items.
#[cfg(feature = "ssr")]
pub fn collect_registered_apps() -> Vec<AppDirectoryItem> {
    use uf_product::AppRegistry;

    let mut items = Vec::new();
    for registration in AppRegistry::auto_discover().iter() {
        items.push(AppDirectoryItem {
            name: registration.name.to_string(),
            slug: registration.id.to_string(),
            description: registration.description.to_string(),
            route_path: registration.route_path.to_string(),
            repository: registration.repository.map(str::to_string),
            crate_name: registration.crate_name.map(str::to_string),
        });
    }
    items
}

#[cfg(not(feature = "ssr"))]
#[allow(dead_code)]
const fn collect_registered_apps() -> Vec<AppDirectoryItem> {
    vec![]
}

/// Legacy non-paginated apps endpoint — kept for backward compatibility.
///
/// **Public by design:** returns in-memory `uf_app!` registration metadata only
/// (name, slug, description, route path). No user data or Valence reads.
///
/// # Errors
///
/// Returns [`ServerFnError`] when the server-fn transport or SSR extractor fails.
/// An empty registry still returns `Ok` with zero items.
#[uf_product_macros::server]
pub async fn get_apps() -> Result<Vec<AppDirectoryItem>, ServerFnError> {
    maybe_simulate_delay().await;
    Ok(collect_registered_apps())
}

/// Paginated apps endpoint.
///
/// Returns a [`Page<AppDirectoryItem>`] using the standard `orbital-paging`
/// over-fetch pattern. Apps come from the in-memory registry: fetch all,
/// sort by name, filter by optional search query, then slice.
///
/// **Public by design:** registry metadata only; no permission gate required.
///
/// # Errors
///
/// Returns [`ServerFnError`] when the server-fn transport or SSR extractor fails.
/// An empty registry or a filter with no matches still returns `Ok` with zero items.
#[uf_product_macros::server]
pub async fn get_apps_page(
    /// Zero-based index of the first app to return.
    offset: u32,
    /// Maximum number of apps to return.
    limit: u32,
    /// Optional case-insensitive search string matched against name and description.
    query: Option<String>,
) -> Result<Page<AppDirectoryItem>, ServerFnError> {
    maybe_simulate_delay().await;
    let mut apps = collect_registered_apps();
    sort_apps_by_name(&mut apps);
    filter_apps_by_query(&mut apps, query.as_deref());
    Ok(page_apps(apps, offset, limit))
}

/// Fetch a single app's overview by its id/slug, or `None` if no app with that id
/// is registered.
///
/// **Public by design:** registry metadata only; no permission gate required.
///
/// # Errors
///
/// Returns [`ServerFnError`] when the server-fn transport or SSR extractor fails.
/// An unknown slug returns `Ok(None)`, not an error.
#[uf_product_macros::server]
pub async fn get_app_overview(
    /// App id/slug to look up.
    app_name: String,
) -> Result<Option<AppOverview>, ServerFnError> {
    maybe_simulate_delay().await;
    let apps = collect_registered_apps();
    Ok(find_app_overview(&apps, &app_name))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_apps() -> Vec<AppDirectoryItem> {
        vec![
            AppDirectoryItem {
                name: "Welcome".into(),
                slug: "welcome".into(),
                description: "Onboarding home".into(),
                route_path: "/welcome".into(),
                repository: None,
                crate_name: None,
            },
            AppDirectoryItem {
                name: "Apps".into(),
                slug: "apps".into(),
                description: "Apps directory".into(),
                route_path: "/apps".into(),
                repository: Some(
                    "https://github.com/unified-field-dev/unified-field-product".into(),
                ),
                crate_name: Some("uf-apps".into()),
            },
            AppDirectoryItem {
                name: "Notifications".into(),
                slug: "notifications".into(),
                description: "Inbox and alerts".into(),
                route_path: "/notifications".into(),
                repository: None,
                crate_name: None,
            },
        ]
    }

    #[test]
    fn sort_apps_by_name_orders_lexicographically_happy_path() {
        let mut apps = sample_apps();
        sort_apps_by_name(&mut apps);
        let names: Vec<_> = apps.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(names, ["Apps", "Notifications", "Welcome"]);
    }

    #[test]
    fn filter_apps_by_query_matches_name_or_description_happy_path() {
        let mut apps = sample_apps();
        filter_apps_by_query(&mut apps, Some("inbox"));
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].slug, "notifications");
    }

    #[test]
    fn filter_apps_by_query_unknown_empty_sad() {
        let mut apps = sample_apps();
        filter_apps_by_query(&mut apps, Some("zz-no-such-app"));
        assert!(
            apps.is_empty(),
            "unknown query should remove all apps, got {}",
            apps.len()
        );
    }

    #[test]
    fn filter_apps_by_query_blank_keeps_all_happy_path() {
        let mut apps = sample_apps();
        filter_apps_by_query(&mut apps, Some("   "));
        assert_eq!(apps.len(), 3);
    }

    #[test]
    fn find_app_overview_resolves_slug_happy_path() {
        let apps = sample_apps();
        let overview = find_app_overview(&apps, "apps").expect("apps slug");
        assert_eq!(overview.name, "Apps");
        assert_eq!(overview.route_path, "/apps");
    }

    #[test]
    fn find_app_overview_unknown_slug_none_sad() {
        let apps = sample_apps();
        assert!(find_app_overview(&apps, "missing").is_none());
    }

    #[test]
    fn page_apps_first_page_reports_total_happy_path() {
        let mut apps = sample_apps();
        sort_apps_by_name(&mut apps);
        let page = page_apps(apps, 0, 2);
        assert_eq!(page.total_count, Some(3));
        assert_eq!(page.items.len(), 2);
    }

    #[test]
    fn page_apps_offset_beyond_end_empty_sad() {
        let apps = sample_apps();
        let page = page_apps(apps, 100, 12);
        assert!(page.items.is_empty());
        assert_eq!(page.total_count, None);
    }

    #[test]
    fn page_apps_maximum_limit_does_not_overflow() {
        let page = page_apps(sample_apps(), 0, u32::MAX);
        assert_eq!(page.items.len(), 3);
        assert_eq!(page.total_count, Some(3));
    }
}
