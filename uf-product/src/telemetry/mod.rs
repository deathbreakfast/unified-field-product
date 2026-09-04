//! Shell Spectra telemetry: page views and appearance-driven theme sync.
//!
//! Mount trackers once under the main product router so navigations emit
//! page-view events and the active theme follows the current app brand seed.
//! Spectra topic schema registration lives in [`crate::spectra_schemas`];
//! appearance preference storage in [`crate::theme`] and [`crate::services`].
//!
//! # Concern → API
//!
//! | Concern | API |
//! |---------|-----|
//! | Track navigations for `uf_app!` products | [`PageViewTracker`], [`UfAppRouteEntry`] |
//! | Map pathname → owning app / brand seed | [`resolve_app_for_path`] |
//! | Keep theme mode/brand in sync on navigation | [`AppearanceThemeController`] |
//! | SSR emit helper | `record_page_view_telemetry` (feature `ssr`) |
//! | Aggregate page views for welcome cards | [`usage`] (feature `ssr`) |
//! | Recent / most-used / popular from Spectra | [`usage::recent_apps_for_viewer`], [`usage::most_used_for_viewer`], [`usage::popular_apps`] |
//!
//! # Example
//!
//! ```rust,ignore
//! use leptos::prelude::*;
//! use uf_product::telemetry::{AppearanceThemeController, PageViewTracker, UfAppRouteEntry};
//!
//! static ROUTES: &[UfAppRouteEntry] = &[UfAppRouteEntry {
//!     app_id: "counter",
//!     app_name: "Counter",
//!     route_prefix: "/counter",
//!     brand_seed: "#1a6f94",
//! }];
//!
//! view! {
//!     <PageViewTracker routes=ROUTES surface="main".to_string() />
//!     <AppearanceThemeController routes=ROUTES />
//! }
//! ```
//!
//! Runnable host: `uf-product/examples/auth_shell_host`.

#[cfg(any(feature = "ssr", feature = "hydrate"))]
pub mod appearance_controller;
// Server-only: pulls in serde_json / spectra-core, which are ssr deps.
#[cfg(feature = "ssr")]
mod events;
#[cfg(any(feature = "ssr", feature = "hydrate"))]
pub mod page_view_tracker;
#[cfg(feature = "ssr")]
mod page_views;
/// Page-view usage aggregators for welcome / product shortcuts.
#[cfg(feature = "ssr")]
pub mod usage;

#[cfg(any(feature = "ssr", feature = "hydrate"))]
pub use appearance_controller::AppearanceThemeController;
#[cfg(any(feature = "ssr", feature = "hydrate"))]
pub use page_view_tracker::{resolve_app_for_path, PageViewTracker, UfAppRouteEntry};
#[cfg(feature = "ssr")]
pub use page_views::record_page_view_telemetry;
