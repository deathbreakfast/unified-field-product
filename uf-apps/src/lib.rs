#![recursion_limit = "256"]
//! Apps directory for Unified Field product hosts.
//!
//! Lists every product app registered with `uf_app!` on a searchable `/apps`
//! index, and opens a detail page with name, description, primary route, and
//! optional repository / docs.rs links.
//!
//! Registration metadata comes from [`uf_product::AppRegistration`] /
//! [`uf_product::AppRegistry`]. Shell chrome that hosts the app-bar Apps control
//! lives in `uf-integrations`.
//!
//! ## Features
//!
//! - **Apps directory routes** — Nested `/apps` and `/apps/:app_name` pages backed by
//!   [`UfAppsRoutes`]. Mount in the host router; metadata comes from `uf_app!` inventory.
//!   [Get started](#getting-started)
//! - **Searchable app index** — Paginated grid with case-insensitive search over registered
//!   app names and descriptions via [`AppsIndexPage`] and [`server::get_apps_page`].
//!   [Get started](#searchable-app-index)
//! - **App-bar Apps launcher** — Stock app-bar Apps button and typeahead dialog. Call
//!   [`ensure_app_bar_linked`] once at host boot. [Get started](#link-app-bar-launcher)
//! - **App detail overview** — Per-app overview card from [`AppDetailPage`] and
//!   [`server::get_app_overview`].
//! - **Apps directory self-registration** — This crate registers itself with `uf_app!`
//!   (`id: "apps"`, route `/apps`) so it appears in the host app inventory.
//!
//! ## Searchable app index
//!
//! [`AppsIndexPage`] binds a search box to [`server::get_apps_page`]. Each call returns
//! an [`orbital_paging::Page`] of [`server::AppDirectoryItem`] rows (name, slug,
//! description, `route_path`, optional `repository` / `crate_name`). The server sorts
//! by name, applies a case-insensitive name/description filter when `query` is
//! non-blank, then slices with the orbital over-fetch pattern.
//!
//! ```rust,ignore
//! use uf_apps::server::{filter_apps_by_query, get_apps_page, AppDirectoryItem};
//!
//! // Same filter the index page uses before pagination:
//! let mut apps: Vec<AppDirectoryItem> = /* from get_apps() or tests */;
//! filter_apps_by_query(&mut apps, Some("beacon"));
//! assert!(apps.iter().all(|a| a.name.to_lowercase().contains("beacon")));
//!
//! // Infinite scroll fetch (what AppsIndexPage passes to OrbitalInfiniteScroll):
//! let page = get_apps_page(0, 12, Some("beacon".into())).await?;
//! ```
//!
//! **Failure modes:** [`server::get_apps_page`] returns [`leptos::prelude::ServerFnError`]
//! only when the server-fn transport or SSR wiring fails. An empty registry,
//! blank search, or no matches still return `Ok` with zero items. The UI shows
//! [`uf_product::components::EmptyState`] when the filtered list is empty.
//!
//! ## App detail overview
//!
//! [`AppDetailPage`] reads `:app_name` from the router and calls
//! [`server::get_app_overview`]. Success yields [`server::AppOverview`]; a missing
//! slug returns `Ok(None)` and the page shows a warning banner. Transport or SSR
//! errors surface as `Err(ServerFnError)` with an error
//! [`uf_product::primitives::MessageBar`].
//!
//! ## Getting started
//!
//! Mount [`UfAppsRoutes`] inside the host Leptos router to expose the searchable
//! `/apps` directory and per-app detail pages. Every `uf_app!` registration in the
//! linked binary appears on the index automatically—no Valence reads or permission
//! gates. Optional app-bar wiring is separate; see [Link app-bar launcher](#link-app-bar-launcher).
//!
//! ### 1. Mount the directory
//!
//! ```rust,ignore
//! use leptos_router::components::Routes;
//! use uf_apps::UfAppsRoutes;
//!
//! view! {
//!     <Routes fallback=|| "not found">
//!         <UfAppsRoutes />
//!     </Routes>
//! }
//! ```
//!
//! Runnable host: `cargo check -p shell-chrome-host --features ssr`.
//!
//! ### 2. Define an app so it appears in the directory
//!
//! Product apps declare metadata with `uf_app!`. The inventory entry is what
//! [`AppsIndexPage`] and the launcher list:
//!
//! ```rust,ignore
//! use leptos::prelude::*;
//! use leptos_router::components::{ParentRoute, Route};
//! use leptos_router::path;
//! use uf_product_macros::uf_app;
//!
//! #[component(transparent)]
//! fn SampleBeaconRoutes() -> impl leptos_router::MatchNestedRoutes + Clone {
//!     view! {
//!         <ParentRoute path=path!("sample-beacon") view=|| view! { <p>"Sample Beacon"</p> }>
//!             <Route path=path!("") view=|| view! { <p>"ok"</p> } />
//!         </ParentRoute>
//!     }
//!     .into_inner()
//! }
//!
//! uf_app! {
//!     name: "Sample Beacon",
//!     id: "sample-beacon",
//!     description: "Teaching app registered with uf_app!",
//!     icon: "Cube",
//!     version: "0.1.0",
//!     routes: SampleBeaconRoutes,
//!     route_path: "/sample-beacon",
//! }
//! ```
//!
//! Inventory-only smoke (no Leptos mount):
//! `cargo run -p uf-product --example uf_app_registration --features ssr`.
//!
//! ## Link app-bar launcher
//!
//! [`ensure_app_bar_linked`] registers the stock [`AppBarAppsButton`] and
//! [`apps_launcher::AppsLauncher`] dialog with the product app bar. Call it once
//! at host boot before rendering shell chrome; it invokes
//! [`uf_product::register_app_bar_utility`] so inventory submissions survive linking.
//!
//! ```rust,ignore
//! use leptos::prelude::*;
//! use uf_apps::{ensure_app_bar_linked, apps_launcher::AppsLauncher};
//! use uf_product::register_app_bar_utility;
//!
//! // Once at host boot, before UnifiedFieldShellLayout renders:
//! ensure_app_bar_linked();
//!
//! let open = RwSignal::new(false);
//! view! {
//!     <AppsLauncher open=open />
//! };
//! assert!(true, "register_app_bar_utility runs inside ensure_app_bar_linked");
//! ```
//!
//! Prefer this path for the stock app bar. To drive the dialog from custom chrome,
//! skip `ensure_app_bar_linked` and mount [`AppsLauncher`] behind your own trigger.
//!
//! ## Examples
//!
//! Mount [`UfAppsRoutes`], optional [`ensure_app_bar_linked`], and a full `uf_app!` per
//! [Getting started](#getting-started) and [Link app-bar launcher](#link-app-bar-launcher).
//! Searchable app index: [`server::filter_apps_by_query`] + [`server::get_apps_page`].
//! Full host + inventory smoke: `shell-chrome-host`, `uf_app_registration`.
//!
//! ## Where to look next
//!
//! - [`UfAppsRoutes`] — nested `/apps` + `/apps/:app_name` routes.
//! - [`AppsIndexPage`] / [`AppDetailPage`] — index grid and overview card.
//! - [`mod@apps_launcher`] — Dialog typeahead ([`AppsLauncher`], [`safe_app_route_path`]).
//! - [`mod@server`] — [`server::get_apps_page`], [`server::get_app_overview`], [`server::AppDirectoryItem`].
//! - [`ensure_help_linked`] — seeded Help spotlight steps; call once so
//!   inventory links into the host binary (see [`uf_help`] authoring ladder).
//! - [`uf_product::AppRegistration`] / [`uf_product::AppRegistry`] — registration contracts.
//! - `uf_product_macros::uf_app` — registration macro (see that crate; smoke:
//!   `cargo run -p uf-product --example uf_app_registration --features ssr`).
//! - `examples/shell-chrome-host` — mounts this crate beside shell chrome
//!   (`cargo check -p shell-chrome-host --features ssr`).

// Page/layout components and `orbital_routes_extract` expansions are UI surface noise;
// crate-root Features / Concern → API tables are the integrator path.
#![allow(missing_docs)]
#![allow(clippy::unused_unit, unused_imports)]
#![deny(clippy::missing_errors_doc)]

use leptos::prelude::*;
use leptos_router::components::{ParentRoute, Route};
use leptos_router::path;
#[cfg(feature = "lazy-routes")]
use leptos_router::Lazy;
use uf_product_macros::uf_app;

mod app_bar_button;
/// App-bar Apps search launcher (Dialog / OverlayDrawer + typeahead).
pub mod apps_launcher;
mod components;
mod help_steps;
mod layout;
#[cfg(feature = "lazy-routes")]
mod lazy_routes;
mod pages;
mod prefetch;
/// Server functions backing the apps directory index/detail pages.
pub mod server;

pub use app_bar_button::{
    ensure_linked as ensure_app_bar_linked, AppBarAppsButton, APP_BAR_UTILITY_ORDER,
};
pub use apps_launcher::{safe_app_route_path, AppsLauncher, AppsLauncherBody, AppsLauncherResult};
pub use layout::AppsLayout;
#[cfg(feature = "lazy-routes")]
pub use lazy_routes::{prefetch_family, AppDetailRoute, AppsIndexRoute};
pub use pages::{AppDetailPage, AppsIndexPage};
pub use prefetch::PrefetchAppFamily;

/// Force-link Help spotlight inventory from this crate.
pub fn ensure_help_linked() {
    help_steps::ensure_help_steps_linked();
}

// Define the Apps application metadata.
uf_app! {
    name: "Apps",
    id: "apps",
    description: "Apps directory and detail pages",
    icon: "📱",
    version: "0.1.0",
    routes: UfAppsRoutes,
    route_path: "/apps",
    repository: "https://github.com/unified-field-dev/unified-field-product",
    crate_name: "uf-apps",
}

/// Apps application routes: index and detail views.
#[allow(missing_docs)]
#[orbital_macros::orbital_routes_extract]
#[component(transparent)]
pub fn UfAppsRoutes() -> impl leptos_router::MatchNestedRoutes + Clone {
    #[cfg(feature = "lazy-routes")]
    {
        view! {
            <ParentRoute path=path!("apps") view=AppsLayout>
                <Route path=path!("") view={Lazy::<AppsIndexRoute>::new()} />
                <Route path=path!(":app_name") view={Lazy::<AppDetailRoute>::new()} />
            </ParentRoute>
        }
        .into_inner()
    }
    #[cfg(not(feature = "lazy-routes"))]
    {
        view! {
            <ParentRoute path=path!("apps") view=AppsLayout>
                <Route path=path!("") view=AppsIndexPage />
                <Route path=path!(":app_name") view=AppDetailPage />
            </ParentRoute>
        }
        .into_inner()
    }
}
