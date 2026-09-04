//! Proc macros for Unified Field product apps: app registration, server functions with
//! operation context, permission manifests, and search source descriptors.
//!
//! Design-system macros (`#[component_doc]`, route extraction) live in `orbital-macros`.
//! Runtime registry types live in `uf-product::routes` and `uf-search-core`.
//!
//! ## Features
//!
//! - **App registration macro** — The `uf_app!` macro registers a product app (id, name,
//!   icon, routes, optional repository metadata) so the shell discovers it and `uf-codegen`
//!   emits route imports. [Get started](#getting-started)
//! - **Product server macro** — The `#[server]` attribute wraps Leptos server functions with
//!   operation-context plumbing and an optional permission gate. [Get started](#server-macro)
//! - **Step-up server gate** — Optional `step_up` / `step_up = "fresh"` on `#[server]` inserts
//!   `uf_product::permissions::require_step_up` after the permission check so Tier A
//!   mutations demand a recent TOTP window (or a handler-supplied fresh code).
//!   [Get started](#step-up-gate)
//! - **Permission manifest derive** — `#[derive(UfPermissionManifest)]` builds a stable
//!   permission manifest from a crate-local enum for route and server-fn gates.
//! - **Search source macro** — `define_search_sources!` registers backend search sources for
//!   the `/search` command palette via inventory descriptors. [Get started](#define-search-sources)
//!
//! ## Getting started
//!
//! The app registration macro collects metadata and (on SSR) submits an
//! [`uf_product::routes::AppRegistration`](https://docs.rs/uf-product/latest/uf_product/routes/struct.AppRegistration.html)
//! entry so the shell and codegen pass can discover your product app.
//!
//! **Prerequisites:** A product-app crate with a Leptos route component type; enable `ssr`
//! on the host that runs inventory collection.
//!
//! ```rust,ignore
//! use uf_product_macros::uf_app;
//!
//! uf_app! {
//!     name: "Counter",
//!     id: "counter",
//!     description: "A simple counter application",
//!     icon: "📊",
//!     version: "0.1.0",
//!     routes: CounterRoutes,
//!     route_path: "/counter",
//!     repository: "https://github.com/example/counter",
//!     crate_name: "counter-app",
//! }
//! ```
//!
//! On success the macro expands to helpers plus `inventory::submit!` with your `routes:` type
//! and `route_path:` prefix. `uf-codegen` scans the same invocation at build time to emit
//! route imports. See `uf-product` example `uf_app_registration` for a runnable inventory
//! path list that includes `/sample-beacon`.
//!
//! ## Server macro
//!
//! The product server macro wraps Leptos `#[server]` with `uf_product::ssr::with_operation`
//! so each server function receives an operation context. With `permission = "…"`, it checks
//! Gauge `actor_can` before the body runs.
//!
//! **Prerequisites:** `ssr` feature on the host; Valence wired for permission-gated fns.
//!
//! ```rust,ignore
//! // Expands to #[server] with operation_context from with_operation
//! #[uf_product_macros::server]
//! pub async fn counter_get() -> Result<i32, ServerFnError> {
//!     let v = uf_product::ssr::valence().await?;
//!     Ok(0)
//! }
//!
//! #[uf_product_macros::server(permission = "counter.admin.view")]
//! pub async fn counter_admin_get() -> Result<i32, ServerFnError> {
//!     Ok(0)
//! }
//! ```
//!
//! Permission-gated calls return [`ServerFnError`](https://docs.rs/leptos/latest/leptos/server_fn/enum.ServerFnError.html)
//! when `actor_can` fails for the named `permission`. Unwrapped fns still receive operation
//! context through `with_operation`.
//!
//! ## Step-up gate
//!
//! Sensitive mutations that already use `permission = "…"` can also demand a recent TOTP
//! sudo window (or a one-shot fresh code) without hand-writing the gate. The `step_up`
//! attribute expands to `uf_product::permissions::require_step_up` after the permission
//! check so UI clients see a stable `STEP_UP:`
//! [`ServerFnError`](https://docs.rs/leptos/latest/leptos/server_fn/enum.ServerFnError.html)
//! prefix when proof is missing.
//!
//! **Prerequisites:** Host with `ssr`, Higgs session, and lepton-auth writing the sudo
//! window (`verify_totp_for_session`). Prefer `step_up` (window mode) for Tier A; use
//! `step_up = "fresh"` when the handler itself calls `lepton_auth::verify_fresh_totp`.
//!
//! ```rust,ignore
//! use leptos::prelude::*;
//!
//! #[uf_product_macros::server(permission = "GaugeAdmin", step_up)]
//! pub async fn grant_role(target: String) -> Result<(), ServerFnError> {
//!     // require_step_up("window") already ran; proceed under the session actor.
//!     let _ = target;
//!     Ok(())
//! }
//!
//! #[uf_product_macros::server(permission = "SecretsReveal", step_up = "fresh")]
//! pub async fn reveal_secret(id: String, totp_code: String) -> Result<String, ServerFnError> {
//!     lepton_auth::verify_fresh_totp(&totp_code).await.map_err(|e| {
//!         ServerFnError::new(format!("STEP_UP:{}", e))
//!     })?;
//!     Ok(id)
//! }
//! ```
//!
//! Window-mode calls return `STEP_UP:step_up_required:` (or `_expired:`) until the
//! operator completes the step-up dialog. Fresh mode always passes the macro gate; the
//! handler must still verify the code. Unknown `step_up` tokens are a compile error.
//!
//! Next: wire `uf_product::permissions::require_step_up` directly when you are not using
//! the attribute, or open lepton-auth **Verify TOTP for a sudo window**.
//!
//! ## Define search sources
//!
//! The search source macro registers one or more backend providers and (on SSR) submits
//! [`SearchSourceDescriptor`](https://docs.rs/uf-search-core/latest/uf_search_core/struct.SearchSourceDescriptor.html)
//! entries so the command palette discovers sources without a hand-written registry list.
//!
//! **Prerequisites:** `ssr` on the host; a type implementing
//! [`uf_search_core::SearchSourceProvider`](https://docs.rs/uf-search-core/latest/uf_search_core/trait.SearchSourceProvider.html).
//!
//! ```rust,ignore
//! use uf_product_macros::define_search_sources;
//!
//! define_search_sources! {
//!     enum AppSearchSourceId {
//!         Apps => {
//!             id: "apps",
//!             label: "Apps",
//!             description: "Search registered apps",
//!             provider: AppsSearchSource
//!         }
//!     }
//! }
//! ```
//!
//! On success each variant expands to a static `SearchSourceDescriptor` with the given `id:`,
//! collected at startup through `inventory::submit!`. Wire the resulting keys into
//! `uf_integrations::SearchSourcePicker` and implement the provider against
//! `uf_search_core::SearchSourceProvider`.
//!
//! ## Examples
//!
//! Start with `uf_app!` registration fields in [Getting started](#getting-started).
//! Inventory path list includes `/sample-beacon` in `uf-product` example
//! `uf_app_registration`.
//!
//! ```bash
//! cargo run -p uf-product --example uf_app_registration --features ssr
//! ```
//!
//! ## Where to look next
//!
//! - [`uf_app`] — expands to app metadata + `inventory::submit!` registration.
//! - [`macro@server`] — SSR-side operation context wrapper around `#[leptos::server]`.
//! - [`define_search_sources!`] — search source ids + SSR descriptors (`uf-search-core`).
//! - `uf-codegen` — build-time scan of `uf_app!` into route import tables.
//! - `uf-product` — runtime `AppRegistration` / guards / permission contracts.

use proc_macro::TokenStream;

mod app_definition;
mod permission_manifest_derive;
mod search_sources;
mod server;

/// Register a product app for Unified Field shell discovery.
///
/// Expands to metadata helpers and (on SSR) an `inventory::submit!` of
/// `uf_product::routes::AppRegistration`. `uf-codegen` also scans this invocation
/// at build time to emit route imports.
///
/// | Field | Required | Notes |
/// |-------|----------|-------|
/// | `id` | yes | String literal; stable app id |
/// | `name` | yes | Display name |
/// | `description` | yes | Short blurb for discovery UI |
/// | `icon` | yes | Icon key or emoji understood by the shell |
/// | `version` | yes | Semver string for display |
/// | `routes` | yes* | Route component type; required for codegen imports |
/// | `route_path` | yes | Primary prefix, e.g. `"/counter"` |
/// | `permission_manifest` | no | Path to an `AppPermissionManifestProvider` |
/// | `brand_seed` | no | Optional `#RRGGBB` override |
/// | `permissions`, `navigation` | no | Accepted for forward compatibility; not expanded yet |
///
/// \* Codegen skips packages that omit `routes`.
///
/// # Examples
///
/// ```rust,ignore
/// use uf_product_macros::uf_app;
///
/// uf_app! {
///     name: "Counter",
///     id: "counter",
///     description: "A simple counter application",
///     icon: "📊",
///     version: "0.1.0",
///     routes: CounterRoutes,
///     route_path: "/counter",
/// }
/// ```
#[proc_macro]
pub fn uf_app(input: TokenStream) -> TokenStream {
    app_definition::expand_uf_app(input.into()).into()
}

/// Wrapper around Leptos `#[server]` with operation context and optional permission gate.
///
/// Without arguments, wraps the async body in `uf_product::ssr::with_operation(fn_name, …)`.
/// With `permission = "…"`, uses `higgs::server_runtime::with_operation` and a Gauge
/// `actor_can` check before the body runs.
///
/// # Examples
///
/// ```rust,ignore
/// #[uf_product_macros::server]
/// pub async fn counter_get() -> Result<i32, ServerFnError> {
///     let v = uf_product::ssr::valence().await?;
///     // …
///     Ok(0)
/// }
///
/// #[uf_product_macros::server(permission = "counter.admin.view")]
/// pub async fn counter_admin_get() -> Result<i32, ServerFnError> {
///     Ok(0)
/// }
/// ```
#[proc_macro_attribute]
pub fn server(attr: TokenStream, input: TokenStream) -> TokenStream {
    server::expand_server(attr.into(), input.into()).into()
}

/// Derive helper for crate-local Unified Field permission manifests.
///
/// Place `#[permission_manifest(domain_key = "…", domain_name = "…", domain_description = "…")]`
/// on the type and `#[permission(name = "…", description = "…")]` on each variant.
///
/// # Examples
///
/// ```rust,ignore
/// use uf_product_macros::UfPermissionManifest;
///
/// #[derive(UfPermissionManifest)]
/// #[permission_manifest(
///     domain_key = "counter_admin",
///     domain_name = "Counter Admin",
///     domain_description = "Administrative actions for the counter app"
/// )]
/// enum CounterPermission {
///     #[permission(name = "counter.admin.view", description = "View counter admin")]
///     ViewAdmin,
///     #[permission(name = "counter.admin.set", description = "Set the counter")]
///     SetCounter,
/// }
/// ```
#[proc_macro_derive(UfPermissionManifest, attributes(permission_manifest, permission))]
pub fn derive_uf_permission_manifest(input: TokenStream) -> TokenStream {
    permission_manifest_derive::expand_derive_permission_manifest(input.into()).into()
}

/// Define search sources and SSR descriptor registrations.
///
/// Each variant needs `id`, `label`, `description`, and a `provider` path implementing
/// `uf_search_core::SearchSourceProvider`. On SSR, expands to `inventory::submit!` of
/// `SearchSourceDescriptor` entries.
///
/// # Examples
///
/// ```rust,ignore
/// use uf_product_macros::define_search_sources;
///
/// define_search_sources! {
///     enum AppSearchSourceId {
///         Apps => {
///             id: "apps",
///             label: "Apps",
///             description: "Search registered apps",
///             provider: AppsSearchSource
///         }
///     }
/// }
/// ```
///
/// Wire the resulting keys into `uf_integrations::SearchSourcePicker` and implement
/// the provider against `uf_search_core::SearchSourceProvider`.
#[proc_macro]
pub fn define_search_sources(input: TokenStream) -> TokenStream {
    search_sources::expand_define_search_sources(input.into()).into()
}
