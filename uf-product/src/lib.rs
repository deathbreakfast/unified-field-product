#![recursion_limit = "256"]
//! # uf-product — Unified Field product shell APIs
//!
//! Leptos product hosts pull session state, route guards, app registration,
//! permission manifests, workspace search contracts, appearance preferences,
//! design-system primitives, and page-view telemetry from this crate.
//!
//! ## Features
//!
//! - **Session context** — Shared [`AuthContext`] signals for the signed-in user across
//!   the Leptos tree. Call once near the host `Router` at boot. [Get started](#session-context-at-boot)
//! - **Authenticated profile hooks** — Read display name, email, and roles without
//!   matching on [`AuthSession`] yourself. [Get started](#read-signed-in-profile)
//! - **Auth state branching** — Match [`AuthSession::Anonymous`] vs authenticated in
//!   chrome and landing pages. [Get started](#branch-auth-state)
//! - **Route guards** — Wrap pages that require sign-in, verified email, or a named
//!   permission. [Get started](#gate-behind-login)
//! - **App registration** — Register product apps for inventory, codegen route tables,
//!   and shell discovery via `uf_app!`. [Get started](#register-app-discovery)
//! - **Permission manifests** — Declare stable permission names for routes, server fns,
//!   and admin tooling. [Get started](#permission-manifests)
//! - **Step-up gate** — Assert a recent TOTP sudo window (or fresh-mode no-op) from product
//!   server functions via [`permissions::require_step_up`]. Prefer the
//!   `#[uf_product_macros::server(..., step_up)]` expansion. [Get started](#require-step-up)
//! - **Workspace content index** — Upsert per-user search rows for AppBar content search
//!   (separate from picker sources in `uf-search-core`). [Get started](#workspace-content-search)
//! - **Appearance preferences** — Load and save light/dark mode and brand colors per user.
//!   [Get started](#appearance-preferences)
//! - **Page-view telemetry** — Emit Spectra page views for registered apps from the main
//!   router. [Get started](#page-view-telemetry)
//! - **Design-system re-exports** — Import [`components`], [`primitives`], [`models`], and
//!   [`nav`] through one dependency path (sourced from `orbital-zone-a`).
//! - **Shell chrome** — App bar, layout, and search UI live in `uf-integrations` (not this crate).
//!
//! ## Getting started
//!
//! Call these in order:
//!
//! 1. [`provide_auth_context`] — put [`AuthContext`] into Leptos context (once, near `Router`).
//! 2. [`init_auth_resource`] — fetch [`get_session`] and write it into that context.
//! 3. In pages: [`use_authenticated_user`] for profile fields, or [`use_auth_state`]
//!    when you need the [`AuthSession::Anonymous`] branch.
//!
//! `rust,ignore` because this needs a Leptos runtime. Copy it into a `ssr`/`hydrate`
//! host. Lib doctests are disabled (`[lib] doctest = false`); use `examples/` and
//! host crates for compile-checked teaching. Compile-checked neighbors:
//! `auth_shell_host` (Axum inventory gate), `examples/shell-chrome-host` (full shell).
//!
//! ```rust,ignore
//! use leptos::prelude::*;
//! use uf_product::{
//!     init_auth_resource, provide_auth_context, use_auth_state, use_authenticated_user,
//!     routes::RequireAuthenticated, AuthSession,
//! };
//!
//! #[component]
//! fn AppRoot() -> impl IntoView {
//!     let auth = provide_auth_context(Default::default());
//!     let auth_resource = init_auth_resource(&auth);
//!     view! {
//!         <SessionChip />
//!         <ProtectedPage />
//!         {move || auth_resource.get().is_some()}
//!     }
//! }
//!
//! #[component]
//! fn SessionChip() -> impl IntoView {
//!     let session = use_auth_state();
//!     view! {
//!         <span>{move || match session.get() {
//!             AuthSession::Anonymous(_) => "Guest",
//!             AuthSession::Authenticated(_) => "Signed in",
//!         }}</span>
//!     }
//! }
//!
//! #[component]
//! fn ProtectedPage() -> impl IntoView {
//!     let user = use_authenticated_user();
//!     view! {
//!         <RequireAuthenticated>
//!             <p>{move || {
//!                 user.get()
//!                     .and_then(|u| u.display_name.clone())
//!                     .unwrap_or_else(|| "you".to_string())
//!             }}</p>
//!         </RequireAuthenticated>
//!     }
//! }
//! ```
//!
//! After sign-in or sign-out, call [`AuthContext::trigger_refresh`] so
//! [`init_auth_resource`] re-runs [`get_session`].
//!
//! ## Session context at boot
//!
//! Once at host boot, before routed pages mount, install shared session signals so every
//! component reads the same [`AuthContext`]. [`provide_auth_context`] inserts the context;
//! [`init_auth_resource`] hydrates it from [`get_session`] on the client after SSR.
//!
//! **Prerequisites:** `ssr` and/or `hydrate` on this crate; host axum-login middleware
//! from `lepton-auth` so [`get_session`] can extract the backend session.
//!
//! ```rust,ignore
//! use leptos::prelude::*;
//! use uf_product::{init_auth_resource, provide_auth_context, use_auth_state, AuthSession};
//!
//! #[component]
//! fn AppRoot() -> impl IntoView {
//!     let auth = provide_auth_context(Default::default());
//!     let auth_resource = init_auth_resource(&auth);
//!     let session = use_auth_state();
//!     view! {
//!         <p>{move || match session.get() {
//!             AuthSession::Anonymous(_) => "Guest",
//!             AuthSession::Authenticated(_) => "Signed in",
//!         }}</p>
//!         {move || auth_resource.get().is_some()}
//!     }
//! }
//! ```
//!
//! On success the session chip renders **Signed in** after hydration; anonymous visitors
//! see **Guest**. [`get_session`] returns a server-fn error when SSR auth extract fails—see
//! [`get_session`] `# Errors`.
//!
//! ## Read signed-in profile
//!
//! [`use_authenticated_user`] exposes display name, email, and roles as an `Option` memo.
//! Use it in page bodies and chrome that should show profile fields without matching on
//! [`AuthSession`] variants.
//!
//! **Prerequisites:** [`provide_auth_context`] and [`init_auth_resource`] from
//! [Session context at boot](#session-context-at-boot).
//!
//! ```rust,ignore
//! use leptos::prelude::*;
//! use uf_product::use_authenticated_user;
//!
//! let user = use_authenticated_user();
//! let label = move || user.get()
//!     .and_then(|u| u.display_name.clone())
//!     .unwrap_or_else(|| "signed out".to_string());
//! view! { <p>{label}</p> }
//! ```
//!
//! Signed-in users see their display name; anonymous or loading states yield
//! `"signed out"`.
//!
//! ## Branch auth state
//!
//! [`use_auth_state`] exposes the full [`AuthSession`] enum when UI must distinguish
//! anonymous visitors from signed-in users—for example app-bar chips and marketing shells
//! that cannot rely on profile fields alone.
//!
//! **Prerequisites:** session context wired at boot (see above).
//!
//! ```rust,ignore
//! use uf_product::{use_auth_state, AuthSession};
//!
//! let session = use_auth_state();
//! let chip = move || match session.get() {
//!     AuthSession::Anonymous(_) => "Guest".to_string(),
//!     AuthSession::Authenticated(user) => user
//!         .display_name
//!         .clone()
//!         .unwrap_or_else(|| user.user_id.clone()),
//! };
//! assert_eq!(chip(), "Guest"); // anonymous fixture
//! ```
//!
//! Anonymous sessions render **Guest**; authenticated sessions show display name or user id.
//!
//! ## Gate behind login
//!
//! [`routes::RequireAuthenticated`] keeps page content hidden until the user signs in.
//! Optional `permission_name` adds a named check (fail-closed until Gauge is wired).
//! Anonymous visitors see the sign-in gate instead of children.
//!
//! **Prerequisites:** session context at boot; host auth middleware. Named permission
//! checks fail closed until Gauge is wired.
//!
//! ```rust,ignore
//! use leptos::prelude::*;
//! use uf_product::routes::RequireAuthenticated;
//!
//! view! {
//!     <RequireAuthenticated permission_name=Some("counter.admin.set")>
//!         <h1>"Counter Admin"</h1>
//!     </RequireAuthenticated>
//! }
//! ```
//!
//! Signed-in users with the permission see the heading; others get the gate UI.
//!
//! ## Register app discovery
//!
//! `uf_app!` registers metadata and (on SSR) submits [`routes::AppRegistration`] to
//! inventory so [`routes::AppRegistry`] and `uf-codegen` can discover routes.
//!
//! **Prerequisites:** `ssr` feature; a Leptos route component for the app.
//!
//! ```rust,ignore
//! use leptos::prelude::*;
//! use leptos_router::components::{ParentRoute, Route};
//! use leptos_router::path;
//! use uf_product_macros::uf_app;
//! use uf_product::routes::AppRegistration;
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
//! let _ = std::hint::black_box(());
//! fn _registration(_: AppRegistration) {}
//! ```
//!
//! Inventory smoke: `cargo run -p uf-product --example uf_app_registration --features ssr`
//! lists `/counter` among discovered route paths.
//!
//! ## Permission manifests
//!
//! Product apps declare a stable permission catalog so routes, server functions, and admin
//! tooling agree on names before Gauge evaluation is wired. [`AppPermissionManifest`]
//! groups [`PermissionSpec`] rows by domain for inventory and runtime gates.
//!
//! **Prerequisites:** none beyond depending on `uf-product`. Runtime checks fail closed
//! until Gauge is wired.
//!
//! ```rust,ignore
//! use uf_product::{
//!     AppPermissionManifest, AppPermissionManifestProvider, PermissionDomainSpec, PermissionSpec,
//! };
//!
//! static PERMS: &[PermissionSpec] = &[PermissionSpec {
//!     name: "counter.admin.set",
//!     description: "Change the global counter value",
//! }];
//! static DOMAINS: &[PermissionDomainSpec] = &[PermissionDomainSpec {
//!     key: "counter_admin",
//!     name: "Counter Admin",
//!     description: "Administrative actions",
//!     permissions: PERMS,
//! }];
//! static MANIFEST: AppPermissionManifest = AppPermissionManifest {
//!     app_id: "counter",
//!     domains: DOMAINS,
//! };
//! struct CounterManifest;
//! impl AppPermissionManifestProvider for CounterManifest {
//!     fn manifest() -> &'static AppPermissionManifest { &MANIFEST }
//! }
//! assert_eq!(CounterManifest::manifest().app_id, "counter");
//! assert_eq!(CounterManifest::manifest().domains.len(), 1);
//! ```
//!
//! The manifest exposes one domain with the `counter.admin.set` permission name.
//! Names are stable for routes and server gates, but Valence rows appear only
//! after a host calls Gauge `sync_permission_manifests` (see that crate’s
//! rustdoc guide **App permission manifest sync**). Runtime checks still fail
//! closed until a `PermissionBackend` is wired.
//!
//! ## Require step-up
//!
//! Product hosts gate Tier A mutations on a recent TOTP sudo window without depending
//! on lepton-auth types in every app crate. Call [`permissions::require_step_up`] after
//! the permission check (or let `#[uf_product_macros::server(..., step_up)]` expand to
//! it) once per sensitive server-fn request under SSR (per-request gate). lepton-auth opens the window with
//! `verify_totp_for_session`; this gate only reads session keys under
//! [`permissions::StepUpMode`].
//!
//! **Prerequisites:** `ssr` feature; Higgs request context; tower-sessions bag holding
//! the sudo window. Call after `require_permission` on each gated mutation.
//!
//! ```rust,ignore
//! use leptos::prelude::*;
//! use uf_product::permissions::{require_step_up, StepUpMode};
//!
//! async fn grant_role_inner() -> Result<(), ServerFnError> {
//!     require_step_up(StepUpMode::Window.as_str()).await?;
//!     Ok(())
//! }
//!
//! async fn break_glass_inner() -> Result<(), ServerFnError> {
//!     // Fresh mode: macro/gate returns Ok; handler must verify the code itself.
//!     require_step_up(StepUpMode::Fresh.as_str()).await?;
//!     Ok(())
//! }
//! ```
//!
//! Window mode returns `STEP_UP:step_up_required:` / `STEP_UP:step_up_expired:` when the
//! bag is missing or stale. Fresh mode always returns `Ok(())` here so handlers can call
//! `lepton_auth::verify_fresh_totp`. Optional hosts install
//! [`permissions::provide_step_up_backend`] once at shell boot.
//!
//! Next: `uf-product-macros` **Step-up gate**, or lepton-auth **Verify TOTP for a sudo window**.
//!
//! ## Workspace content search
//!
//! [`workspace_search`] maintains a per-user content index for AppBar search in
//! `uf-integrations`. Source models register a Valence `SideEffect` to upsert index rows.
//!
//! **Prerequisites:** `ssr` feature; Valence schema for
//! [`workspace_search::SearchDocumentWriter`].
//!
//! ```rust,ignore
//! use valence::SideEffect;
//! use uf_product::workspace_search::{SearchDocumentWriter, query, WorkspaceSearchError};
//!
//! // Register SideEffect on your source model (see workspace_search::demo).
//! // let draft = SearchDocumentDraft { /* user, title, link, … */ };
//! // SearchDocumentWriter::upsert(&valence, draft).await?;
//! let hits = query(&valence, "title", 10).await;
//! assert!(matches!(hits, Ok(_) | Err(WorkspaceSearchError::Unauthenticated)));
//! ```
//!
//! Successful queries return matching hits; unauthenticated callers get an
//! `Unauthenticated` variant from [`workspace_search::WorkspaceSearchError`].
//!
//! ## Appearance preferences
//!
//! Signed-in users persist light/dark mode and brand colors through server functions while
//! a local mirror speeds first paint. [`provide_appearance_context`] holds reactive prefs;
//! [`save_my_appearance`] writes the Valence row when settings change.
//!
//! **Prerequisites:** `ssr`/`hydrate`; session context; Valence user appearance schema.
//!
//! ```rust,ignore
//! use uf_product::{
//!     provide_appearance_context, save_my_appearance, AppearancePreferences, init_appearance_resource,
//! };
//!
//! let appearance = provide_appearance_context(Default::default());
//! init_appearance_resource(&appearance);
//! let prefs = AppearancePreferences {
//!     color_mode: "dark".into(),
//!     brand_source: "product".into(),
//!     brand_seed_color: None,
//! };
//! let save_fn = save_my_appearance;
//! assert_eq!(prefs.color_mode, "dark");
//! let _ = std::hint::black_box(save_fn);
//! ```
//!
//! After a successful save the user's stored `color_mode` matches the submitted value.
//!
//! ## Page-view telemetry
//!
//! [`PageViewTracker`] listens to router navigations and emits Spectra page views using
//! the codegen route table from registered apps.
//!
//! **Prerequisites:** `ssr`/`hydrate`; `spectra-telemetry` when exporting to Spectra;
//! mount inside the host `<Routes>` tree.
//!
//! ```rust,ignore
//! use leptos::prelude::*;
//! use uf_product::telemetry::page_view_tracker::{PageViewTracker, resolve_app_for_path, UfAppRouteEntry};
//!
//! static ROUTES: &[UfAppRouteEntry] = &[UfAppRouteEntry {
//!     app_id: "counter",
//!     app_name: "Counter",
//!     route_prefix: "/counter",
//!     brand_seed: "#1a6f94",
//! }];
//! let entry = resolve_app_for_path("/counter/settings", ROUTES);
//! assert_eq!(entry.app_id, "counter");
//! view! { <PageViewTracker routes=ROUTES surface="main".to_string() /> }
//! ```
//!
//! Navigating under `/counter` resolves `app_id` **counter** and records a page view.
//!
//! ## Feature flags
//!
//! | Feature | Effect |
//! |---------|--------|
//! | `default` | Enables `preview` (component preview registrations). |
//! | `preview` | Re-exports Orbital preview APIs from `orbital-zone-a`. |
//! | `ssr` | Server fns, session bridge, workspace search writers, app registry, Spectra hooks. |
//! | `hydrate` | Client graph for session, appearance, workspace search query, telemetry. |
//! | `spectra-telemetry` | Pulls Spectra + typed page-view topics (requires `ssr`). |
//! | `permissions` | Placeholder until Gauge is git-standalone; gates stay fail-closed. |
//! | `db-sqlite` / `db-hybrid` | Valence backend for appearance and search persistence. |
//!
//! ## Examples
//!
//! Start with `provide` → hydrate → [`use_authenticated_user`] / [`use_auth_state`] in
//! [Getting started](#getting-started). Same flow on [`provide_auth_context`],
//! [`use_authenticated_user`], and [`use_auth_state`].
//!
//! Workspace `uf-product/examples/`: `uf_app_registration` (`uf_app!`), `app_route_paths`,
//! `auth_shell_host` (Axum inventory gate). Nested UI hosts in workspace `examples/`:
//! `shell-chrome-host`, `component-preview-host`.
//!
//! ```bash
//! cargo run -p uf-product --example uf_app_registration --features ssr
//! cargo run -p uf-product --example auth_shell_host --features ssr
//! ```
//!
//! ## Where to look next
//!
//! - [`provide_auth_context`] / [`use_authenticated_user`] / [`use_auth_state`] — session.
//! - [`permissions`] — permission manifest contracts.
//! - [`routes`] — app registration + route guards.
//! - `uf-integrations` — shell app bar, `WorkspaceSearch`, and `SearchSourcePicker`.
//! - [`workspace_search`] — per-user content index (SideEffect/Iter writers + query).
//! - `uf-search-core` — picker DTOs/registry (also via [`search_sources`]).
//! - `uf-product-macros` / `uf-codegen` — `uf_app!` registration and build-time route discovery.

// Narrow allow: design-system / Spectra / inventory re-exports and generated SSR
// modules expand many public items without local rustdoc. Prefer documenting new
// product-owned APIs at the item; do not widen this allow for ordinary new modules.
#![allow(missing_docs)]
#![deny(clippy::missing_errors_doc)]

pub use orbital_zone_a::context;
pub use orbital_zone_a::models;
pub use orbital_zone_a::primitives;

pub mod app_bar_menu_extras;
pub mod app_bar_utilities;
pub mod auth_dialog;
pub mod components;
pub mod nav;
pub mod paths;
pub mod permissions;
pub mod routes;
pub mod search_sources;
pub mod services;
/// Host session → [`AuthSession`] bridge (`get_session` / `init_auth_resource`).
#[cfg(any(feature = "hydrate", feature = "ssr"))]
pub mod session;
pub mod theme;
#[cfg(any(feature = "ssr", feature = "hydrate"))]
pub mod workspace_search;

#[cfg(any(feature = "hydrate", feature = "ssr", feature = "preview"))]
pub use orbital_zone_a::preview;

pub use permissions::AppPermissionManifest;
pub use permissions::AppPermissionManifestProvider;
pub use permissions::PermissionDomainSpec;
pub use permissions::PermissionEnum;
pub use permissions::PermissionSpec;

#[cfg(feature = "ssr")]
pub use quark::inventory;
#[cfg(feature = "ssr")]
pub use routes::AppRegistration;
#[cfg(feature = "ssr")]
pub use routes::AppRegistry;

pub use app_bar_menu_extras::{
    app_bar_dark_mode_bind, provide_app_bar_menu_extras, use_app_bar_menu_extras,
    AppBarAppearanceMenuItems, AppBarCompactMenuExtras, AppBarMenuExtrasInjection,
};
pub use app_bar_utilities::{
    collect_app_bar_utilities, register_app_bar_utility, AppBarUtilityContribution,
};
pub use auth_dialog::{
    provide_auth_dialog_controller, use_auth_dialog_controller, AuthDialogController,
    AuthDialogIntent,
};
/// Shared session signals. Provide once with [`provide_auth_context`]; pages read
/// [`use_authenticated_user`] or [`use_auth_state`] instead of constructing this.
///
/// After sign-in or sign-out, call [`AuthContext::trigger_refresh`].
pub use context::AuthContext;

/// Insert [`AuthContext`] into Leptos context. Call once near the host `Router`.
///
/// Pair with [`init_auth_resource`] so [`get_session`] fills the session signal.
///
/// # Examples
///
/// ```rust,ignore
/// use uf_product::{init_auth_resource, provide_auth_context};
///
/// let auth = provide_auth_context(Default::default());
/// let _session = init_auth_resource(&auth);
/// ```
pub use context::provide_auth_context;

/// Handle for [`AuthContext::trigger_refresh`] and `session_loaded`.
///
/// # Panics
///
/// Panics if [`provide_auth_context`] has not run in an ancestor.
pub use context::use_auth_context;

/// Full [`AuthSession`] memo: match [`AuthSession::Anonymous`] vs
/// [`AuthSession::Authenticated`].
///
/// Prefer [`use_authenticated_user`] when you only need profile fields.
///
/// # Examples
///
/// ```rust,ignore
/// use uf_product::{use_auth_state, AuthSession};
///
/// let session = use_auth_state();
/// let label = move || match session.get() {
///     AuthSession::Anonymous(_) => "Guest".to_string(),
///     AuthSession::Authenticated(user) => user
///         .display_name
///         .clone()
///         .unwrap_or_else(|| user.user_id.clone()),
/// };
/// ```
pub use context::use_auth_state;

/// Signed-in profile memo (`None` while anonymous or still loading).
///
/// Use this for display name, email, and roles. Pair with
/// [`routes::RequireAuthenticated`] so the `None` branch is not the page body.
///
/// # Examples
///
/// ```rust,ignore
/// use leptos::prelude::*;
/// use uf_product::use_authenticated_user;
///
/// let user = use_authenticated_user();
/// view! {
///     <p>{move || {
///         user.get()
///             .and_then(|u| u.display_name.clone())
///             .unwrap_or_else(|| "signed out".to_string())
///     }}</p>
/// }
/// ```
pub use context::use_authenticated_user;
pub use models::auth::{AnonymousUser, AuthSession, AuthenticatedUser};
pub use routes::{provide_access_gate_state, use_access_gate_active, AccessGateActive};

#[cfg(any(feature = "hydrate", feature = "ssr"))]
pub use session::{get_session, init_auth_resource};

#[cfg(any(feature = "hydrate", feature = "ssr", feature = "preview"))]
pub use preview::{collect_preview_registrations, PreviewRegistration};

#[cfg(any(feature = "hydrate", feature = "ssr"))]
pub use crate::theme::{
    apply_appearance_preferences, read_local_appearance, write_local_appearance,
    UF_SHELL_BRAND_SEED,
};
#[cfg(any(feature = "hydrate", feature = "ssr"))]
pub use services::{
    get_my_appearance, init_appearance_resource, provide_appearance_context, save_my_appearance,
    use_appearance_preferences, AppearanceContext, AppearanceData, AppearancePreferences,
    APPEARANCE_STORAGE_KEY, PRODUCT_BRAND_PRESETS,
};
#[cfg(any(feature = "hydrate", feature = "ssr"))]
pub use telemetry::AppearanceThemeController;
#[cfg(any(feature = "hydrate", feature = "ssr"))]
pub use telemetry::PageViewTracker;

pub use orbital_theme::ThemeMode;
pub use orbital_zone_a::{
    hide_boot_loader, OrbitalBootLoaderHeadAssets, OrbitalBootOverlay, OrbitalFirstPaintHeadAssets,
};

pub mod tokens;
pub use orbital_zone_a::{orbital_shell, OrbitalTemplate};

#[cfg(feature = "ssr")]
pub mod embedded_surreal;
#[cfg(feature = "ssr")]
pub mod generated;
#[cfg(feature = "ssr")]
mod schemas;
#[cfg(feature = "ssr")]
pub mod spectra_schemas;
#[cfg(all(feature = "ssr", feature = "spectra-telemetry"))]
pub mod spectra_topics;
#[cfg(feature = "ssr")]
pub mod ssr;
#[cfg(any(feature = "ssr", feature = "hydrate"))]
pub mod telemetry;
