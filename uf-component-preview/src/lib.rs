#![recursion_limit = "256"]
#![allow(missing_docs)]
//! Component library and preview host for Orbital UI in Unified Field product builds.
//!
//! Serves a registry-driven catalog at `/orbital/{slug}` for previewable components
//! from Orbital leaf crates (via
//! [`orbital_primitives::preview::collect_all_preview_registrations`]), plus Unified
//! Field-specific previews and layout examples. Integrators mount this crate in dev or
//! staging hosts; production shell chrome, auth, and search wiring live outside it.
//!
//! ## Features
//!
//! - **Orbital preview catalog** — Nested `/orbital` routes backed by [`OrbitalComponentRoutes`].
//!   Registry-driven slug pages, introduction, and catalog shell chrome. Mount once in a
//!   dev or staging host router (typically not production). [Get started](#getting-started)
//! - **Preview registry merge** — [`preview::collect_preview_registrations`] merges Orbital baseline,
//!   UF manual pages, and teaching examples for slug routing via [`preview::PreviewSlugPage`].
//! - **Catalog chrome** — [`preview::PreviewCatalogShell`], [`preview::PreviewCatalogNav`], and
//!   [`preview::PreviewCatalogSearch`] compose the left nav and search for the catalog.
//! - **Teaching example** — [`components::examples::DemoStatusPill`] demonstrates `#[component_doc]`
//!   registration end to end.
//!
//! ## Feature flags
//!
//! | Feature | Effect |
//! |---------|--------|
//! | `preview` | Marks this crate as a preview consumer of Orbital / uf-product preview APIs. |
//! | `hydrate` / `ssr` | Leptos client/server split. Zone previews are host-merged via [`preview::extend_registrations`]. |
//!
//! Hosts that want History Timeline and Tag Catalog Picker in the catalog call
//! `extend_registrations` with each owning crate's `preview::all()` at startup once
//! those crates are available from `unified-field-dev`.
//!
//! ## Getting started
//!
//! [`OrbitalComponentRoutes`] exposes the nested Leptos route tree for the Orbital catalog at
//! `/orbital`. Mount it inside the host `<Routes>` in dev or staging builds so designers and
//! engineers can browse registry-driven component previews without shipping catalog routes in
//! production.
//!
//! **Prerequisites:** `ssr` and/or `hydrate` on `uf-component-preview` and host deps; linked
//! Orbital preview registrations from leaf crates when using the full catalog.
//!
//! ```rust,ignore
//! use leptos_router::components::Routes;
//! use uf_component_preview::OrbitalComponentRoutes;
//!
//! view! {
//!     <Routes fallback=|| "not found">
//!         <OrbitalComponentRoutes />
//!     </Routes>
//! }
//! ```
//!
//! On success the host serves `/orbital` (introduction) and `/orbital/{slug}` registry pages.
//! Runnable reference: `cargo check -p component-preview-host --features ssr`.
//!
//! ## Examples
//!
//! Mount [`OrbitalComponentRoutes`] per [Getting started](#getting-started). Teaching
//! `#[component_doc]` widget (`demo-status-pill`) in [`components::examples`]. Full
//! `/orbital` host: `examples/component-preview-host`. Zone preview pages are merged
//! by the host via [`preview::extend_registrations`] when those crates are wired.
//!
//! ```bash
//! cargo check -p component-preview-host --features ssr
//! ```
//!
//! ## Where to look next
//!
//! - [`OrbitalComponentRoutes`] — the `/orbital` route tree.
//! - [`preview`] — the registry, catalog shell, and slug-routed preview page.
//! - [`pages`] — the introduction page and top-level dev layout.
//! - `examples/component-preview-host` — mounts this catalog in-workspace.
//! - [`components::examples`] — teaching `#[component_doc]` widget (`demo-status-pill`).

use uf_product_macros::uf_app;

mod lazy_routes;

/// Preview-only components: hand-written component previews and layout examples.
pub mod components;
/// Top-level pages: the introduction page and the dev shell layout.
pub mod pages;
/// The preview registry, catalog shell chrome, and slug-routed preview page.
pub mod preview;
/// The `/orbital` route tree.
pub mod routes;

// Re-export route components and generated paths for easy use
pub use lazy_routes::prefetch_family;
pub use routes::paths;
pub use routes::OrbitalComponentRoutes;
/// Back-compat alias for hosts that still import [`OrbitalDevRoutes`].
pub use routes::OrbitalComponentRoutes as OrbitalDevRoutes;

// Define the Orbital component preview application metadata.
uf_app! {
    name: "Orbital",
    id: "orbital",
    description: "Component library and preview tool for Orbital UI components",
    icon: "🧩",
    version: "0.1.0",
    routes: OrbitalComponentRoutes,
    route_path: "/orbital",
}
