#![recursion_limit = "256"]
//! Signed-in welcome landing page for Unified Field product hosts.
//!
//! Authenticated users land on `/welcome` and see featured apps plus recent /
//! most-used / popular shortcuts from Spectra page views. Operators with
//! `WelcomeAdmin` manage featured apps at `/welcome/admin`.
//!
//! Host session middleware and shell chrome live outside this crate. Page-view
//! emission uses `uf_product::telemetry::PageViewTracker`.
//!
//! ## Features
//!
//! - **Welcome landing routes** — Nested `/welcome` and `/welcome/admin` pages with
//!   usage cards backed by Spectra page views. Mount once in the host router and wire
//!   [`uf_product::PageViewTracker`] so recent, popular, and most-used cards populate.
//!   [Get started](#getting-started)
//! - **Featured apps admin** — Valence-backed catalog for the Featured card. Operators
//!   with `WelcomeAdmin` add, remove, and reorder rows at `/welcome/admin`.
//!   [Get started](#featured-admin)
//! - **Welcome shell layout** — [`WelcomeLayout`] wraps shell chrome and requires sign-in
//!   before child routes render.
//! - **Help spotlight inventory** — Welcome-route help steps register through inventory;
//!   call [`ensure_help_linked`] once at host boot so the linker retains them.
//! - **Permission manifest** — [`permissions::WelcomePermission`] declares the
//!   `WelcomeAdmin` capability for featured curation.
//!
//! ## Feature flags
//!
//! | Feature | Effect |
//! |---------|--------|
//! | `lazy-routes` (default) | WASM-split welcome + admin route views; use [`prefetch_family`] |
//! | `admin-permissions` | Gauge-backed `WelcomeAdmin` gate on `/welcome/admin` |
//! | `ssr` / `hydrate` | Server and client Leptos graphs |
//!
//! ## Getting started
//!
//! [`UfWelcomeRoutes`] exposes nested Leptos routes for the signed-in welcome page and
//! the featured-apps admin UI. Mount it inside the host `<Routes>` and place
//! [`uf_product::PageViewTracker`] beside the router so Spectra receives page-view events for the
//! usage cards on `/welcome`.
//!
//! **Prerequisites:** `ssr` and/or `hydrate` on `uf-welcome` and host deps; signed-in
//! session from `uf-product`; `Arc<Spectra>` in SSR context for usage aggregators.
//!
//! ```rust,ignore
//! use leptos_router::components::Routes;
//! use uf_product::telemetry::PageViewTracker;
//! use uf_welcome::UfWelcomeRoutes;
//!
//! view! {
//!     <PageViewTracker routes=ROUTES surface="main".to_string() />
//!     <Routes fallback=|| "not found">
//!         <UfWelcomeRoutes />
//!     </Routes>
//! }
//! ```
//!
//! On success the host serves `/welcome` with featured, recent, most-used, and popular
//! cards once page views flow into Spectra. Runnable reference:
//! `cargo check -p shell-chrome-host --features ssr`.
//!
//! ## Featured admin
//!
//! [`WelcomeAdminPage`] renders the `/welcome/admin` UI for curating featured apps.
//! Server mutations require `WelcomeAdmin` from [`permissions::WelcomePermission`]
//! (enable `admin-permissions` for Gauge enforcement). Domain errors surface as
//! [`featured::FeaturedError`] before server functions map them to transport errors.
//!
//! **Prerequisites:** `ssr` on `uf-welcome`; System Valence for catalog writes;
//! registered apps in [`uf_product::AppRegistry`]; signed-in operator with `WelcomeAdmin`.
//!
//! ```rust,ignore
//! use uf_welcome::featured::{add, FeaturedError};
//! use uf_welcome::WelcomeAdminPage;
//!
//! // WelcomeAdmin permission gates server fns (see permissions::WelcomePermission).
//! let err = add(&session_valence, "zz-unknown-app", 0).await.unwrap_err();
//! assert!(matches!(err, FeaturedError::UnknownApp { .. }));
//!
//! view! { <WelcomeAdminPage /> }
//! ```
//!
//! On success promoted apps appear on the welcome Featured card ordered by ordinal.
//! Deep dive: `cargo test -p uf-welcome --features ssr --test featured_service_integ`.
//!
//! ## Examples
//!
//! Mount [`UfWelcomeRoutes`] + [`uf_product::PageViewTracker`] per
//! [Getting started](#getting-started). Featured admin: `WelcomeAdmin` gate +
//! [`featured::FeaturedError`] in [Featured admin](#featured-admin). Shell mount + admin
//! integ: `examples/shell-chrome-host`, `tests/featured_service_integ.rs`.
//!
//! ```bash
//! cargo check -p shell-chrome-host --features ssr
//! cargo test -p uf-welcome --features ssr --test featured_service_integ
//! ```
//!
//! ## Where to look next
//!
//! - [`UfWelcomeRoutes`] — nested routes for welcome + admin.
//! - [`welcome`] — pages, cards, featured service, server fns.
//! - `uf_product::telemetry::usage` — Spectra aggregation helpers.
//! - `examples/shell-chrome-host` — mounts this crate beside shell chrome.

#![allow(missing_docs)]
#![allow(clippy::unused_unit, unused_imports)]
#![deny(clippy::missing_errors_doc)]

use leptos::prelude::*;
#[cfg(feature = "lazy-routes")]
use leptos_router::Lazy;
use leptos_router::{
    components::{Outlet, ParentRoute, Route},
    path,
};
use uf_product_macros::uf_app;

mod help_steps;
#[cfg(feature = "lazy-routes")]
mod lazy_routes;
/// Welcome permission manifest (`WelcomeAdmin`).
pub mod permissions;
#[allow(clippy::too_long_first_doc_paragraph)]
pub mod welcome;

#[cfg(feature = "ssr")]
pub mod embedded_surreal;
#[cfg(feature = "ssr")]
pub mod generated;
#[cfg(feature = "ssr")]
pub mod privacy_policies;
#[cfg(feature = "ssr")]
mod schemas;

#[cfg(feature = "lazy-routes")]
pub use lazy_routes::{prefetch_family, WelcomeAdminPageRoute, WelcomePageRoute};
#[cfg(feature = "ssr")]
pub use welcome::featured;
#[cfg(feature = "ssr")]
pub use welcome::server::E2E_WELCOME_ADMIN_SESSION_KEY;
pub use welcome::{WelcomeAdminPage, WelcomeLayout, WelcomePage};

/// Force-link Help spotlight inventory from this crate.
pub fn ensure_help_linked() {
    help_steps::ensure_help_steps_linked();
}

uf_app! {
    name: "Welcome",
    id: "welcome",
    description: "Signed-in welcome landing for Unified Field product hosts",
    icon: "👋",
    version: "0.1.0",
    routes: UfWelcomeRoutes,
    route_path: "/welcome",
    repository: "https://github.com/unified-field-dev/unified-field-product",
    permission_manifest: permissions::WelcomePermission,
}

/// Welcome application routes (`/welcome` and `/welcome/admin`).
#[component(transparent)]
pub fn UfWelcomeRoutes() -> impl leptos_router::MatchNestedRoutes + Clone {
    #[cfg(feature = "lazy-routes")]
    {
        view! {
            <ParentRoute path=path!("") view=Outlet>
                <ParentRoute path=path!("welcome") view=WelcomeLayout>
                    <Route path=path!("") view={Lazy::<WelcomePageRoute>::new()} />
                    <Route path=path!("admin") view={Lazy::<WelcomeAdminPageRoute>::new()} />
                </ParentRoute>
            </ParentRoute>
        }
        .into_inner()
    }
    #[cfg(not(feature = "lazy-routes"))]
    {
        view! {
            <ParentRoute path=path!("") view=Outlet>
                <ParentRoute path=path!("welcome") view=WelcomeLayout>
                    <Route path=path!("") view=WelcomePage />
                    <Route path=path!("admin") view=WelcomeAdminPage />
                </ParentRoute>
            </ParentRoute>
        }
        .into_inner()
    }
}
