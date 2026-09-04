//! Route registration, redirect hygiene, and auth-aware route guards.
//!
//! Apps register metadata through `AppRegistration` and the inventory-backed
//! `AppRegistry` on SSR builds. Runtime UI gates such as [`RequireAuthenticated`]
//! protect pages that need sign-in, verified email, or a specific permission.
//!
//! Host axum-login middleware lives in lepton-auth. Shell chrome and left nav
//! are composed in `uf-integrations`. Macro expansion (`uf-product-macros`) and
//! build-time scan (`uf-codegen`) feed registration into this module.
//!
//! Named permission checks currently fail closed (Gauge not wired yet).
//!
//! # Concern → API
//!
//! | Concern | API |
//! |---------|-----|
//! | Gate page content behind login / permission | [`RequireAuthenticated`] |
//! | Tell Help a sign-in (or other) gate is showing | [`provide_access_gate_state`], [`AccessGateActive`] |
//! | Discover apps registered by `uf_app!` (SSR) | [`AppRegistration`], [`AppRegistry`] |
//! | Parse safe redirect targets from referer | [`parse_referer_from_search`], [`sanitize_referer_path`] |
//! | Auth route + return path when no shell dialog | [`auth_signin_href`], [`auth_signup_href`] |
//!
//! # Example
//!
//! ```rust,ignore
//! use leptos::prelude::*;
//! use uf_product::routes::RequireAuthenticated;
//!
//! #[component]
//! fn CounterAdminPage() -> impl IntoView {
//!     view! {
//!         <RequireAuthenticated permission_name=Some("counter.admin.set")>
//!             <h1>"Counter Admin"</h1>
//!         </RequireAuthenticated>
//!     }
//! }
//! ```
//!
//! Full `uf_app!` registration (inventory + codegen route import when `routes:` is set):
//!
//! ```rust,ignore
//! use leptos::prelude::*;
//! use leptos_router::components::{ParentRoute, Route};
//! use leptos_router::path;
//! use uf_product_macros::uf_app;
//!
//! #[component(transparent)]
//! fn CounterRoutes() -> impl leptos_router::MatchNestedRoutes + Clone {
//!     view! {
//!         <ParentRoute path=path!("counter") view=|| view! { <p>"Counter"</p> }>
//!             <Route path=path!("") view=|| view! { <p>"ok"</p> } />
//!         </ParentRoute>
//!     }
//!     .into_inner()
//! }
//!
//! uf_app! {
//!     name: "Counter",
//!     id: "counter",
//!     description: "Realtime shared counter example",
//!     icon: "NumberSymbolSquare24Regular",
//!     version: "0.1.0",
//!     routes: CounterRoutes,
//!     route_path: "/counter",
//! }
//! ```
//!
//! Inventory-only smoke (no Leptos mount): `cargo run -p uf-product --example uf_app_registration --features ssr`.
//! Route path listing: `app_route_paths`. Axum middleware gate: `auth_shell_host`.

mod access_gate;
mod referer;
#[cfg(feature = "ssr")]
mod registration;
mod require_authenticated;

pub use access_gate::{provide_access_gate_state, use_access_gate_active, AccessGateActive};
pub use referer::{
    auth_path_with_referer, auth_signin_href, auth_signup_href, parse_referer_from_search,
    sanitize_referer_path,
};
#[cfg(feature = "ssr")]
pub use registration::{get_all_app_route_paths, AppRegistration, AppRegistry};
pub use require_authenticated::{authenticated_route_condition, RequireAuthenticated};
