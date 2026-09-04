#![recursion_limit = "256"]
//! Shell integrations (app bar, search picker, layout). Auth menu is host-provided
//! via a slot or Leptos context ([`HostAuthMenu`]). Trailing utilities are a generic
//! children slot (default = optional product offerings via inventory).
//!
//! Session loading and route guards live in `uf-product`. Auth UI widgets come
//! from `lepton_shell::AppBarUserMenu`. Help, Apps, and Appearance widgets live
//! in sibling offering crates. Search provider registry and Valence queries live
//! in `uf-search-core`.
//!
//! ## Features
//!
//! - **Shell layout** — [`UnifiedFieldShellLayout`] wraps Orbital's layout with app-bar
//!   and left-nav slots for product shells. [Get started](#shell-layout)
//! - **Auth menu slot** — [`provide_shell_auth_menu`] and [`HostAuthMenu`] inject a
//!   host-provided sign-in menu without hard-depending on auth crates.
//!   [Get started](#auth-menu-slot)
//! - **Picker search combobox** — [`SearchSourcePicker`] is a reusable multi-source
//!   combobox for pickers backed by `uf-search-core` (separate from AppBar content index).
//!   [Get started](#search-source-picker)
//! - **AppBar workspace search** — [`WorkspaceSearch`] and [`WorkspaceSearchMobileTrigger`]
//!   query the per-user workspace content index from the app bar. [Get started](#workspace-search)
//! - **App bar** — [`UnifiedFieldAppBar`] composes branding, breadcrumbs
//!   ([`BreadcrumbLink`]), workspace search slots, and [`AppBarUtilities`].
//! - **Notification bell slot** — [`provide_shell_notification_bell`] overrides;
//!   with `offering-notifications`, [`HostNotificationBell`] falls back to
//!   inventory from a linked `uf-notifications` crate.
//! - **Placeholder pages** — [`UnifiedFieldComingSoonPage`], [`coming_soon_fill_for_path`],
//!   and [`UnifiedFieldNotFoundPage`] for routes that are not built yet.
//!
//! ## Shell layout
//!
//! [`UnifiedFieldShellLayout`] is the stock product shell wrapper: app bar slot, left nav,
//! and page body. Use it when you want Orbital chrome with Unified Field app-bar utilities
//! rather than hand-rolling layout primitives.
//!
//! **Prerequisites:** `ssr` and/or `hydrate` on this crate.
//!
//! ```rust,ignore
//! use leptos::prelude::*;
//! use uf_integrations::{ShellAppBar, UnifiedFieldAppBar, UnifiedFieldShellLayout};
//!
//! #[component]
//! fn AppShell(children: Children) -> impl IntoView {
//!     view! {
//!         <UnifiedFieldShellLayout>
//!             <ShellAppBar slot>
//!                 <UnifiedFieldAppBar app_name="My App".to_string() />
//!             </ShellAppBar>
//!             {children()}
//!         </UnifiedFieldShellLayout>
//!     }
//! }
//! ```
//!
//! On success the host renders [`UnifiedFieldAppBar`] chrome around routed pages.
//!
//! ## Auth menu slot
//!
//! [`provide_shell_auth_menu`] registers a factory for [`HostAuthMenu`]. Call it **once at
//! host boot** (before routed pages mount) so [`ShellAuthMenu`] can render your
//! `lepton_shell::AppBarUserMenu` inside [`UnifiedFieldAppBar`].
//!
//! **Prerequisites:** `ssr` and/or `hydrate` on this crate; session context from
//! `uf-product` when the menu should reflect sign-in state.
//!
//! ```rust,ignore
//! use leptos::prelude::*;
//! use lepton_shell::AppBarUserMenu;
//! use uf_integrations::{provide_shell_auth_menu, HostAuthMenu, ShellAuthMenu, UnifiedFieldAppBar};
//!
//! // Once at host boot, before routed pages mount:
//! provide_shell_auth_menu(|| view! { <AppBarUserMenu /> });
//!
//! #[component]
//! fn AppBarWithAuth() -> impl IntoView {
//!     view! {
//!         <UnifiedFieldAppBar app_name="My App".to_string()>
//!             <ShellAuthMenu slot:auth_menu>
//!                 <HostAuthMenu />
//!             </ShellAuthMenu>
//!         </UnifiedFieldAppBar>
//!     }
//! }
//! ```
//!
//! On success the app bar shows your `AppBarUserMenu` in the auth slot. Omit
//! [`provide_shell_auth_menu`] when the host supplies auth chrome another way;
//! [`HostAuthMenu`] then renders nothing.
//!
//! ## Getting started
//!
//! Full shell recipe combining layout, auth menu, and optional notification bell:
//!
//! ```rust,ignore
//! use leptos::prelude::*;
//! use lepton_shell::AppBarUserMenu;
//! use uf_integrations::{
//!     provide_shell_auth_menu, HostAuthMenu, ShellAppBar, ShellAuthMenu, UnifiedFieldAppBar,
//!     UnifiedFieldShellLayout,
//! };
//!
//! // Once at host boot, before routed pages mount:
//! provide_shell_auth_menu(|| view! { <AppBarUserMenu /> });
//! // With `offering-notifications` + a linked `uf-notifications` dep, HostNotificationBell
//! // fills via inventory. Call provide_shell_notification_bell only to override.
//!
//! #[component]
//! fn AppShell(children: Children) -> impl IntoView {
//!     view! {
//!         <UnifiedFieldShellLayout>
//!             <ShellAppBar slot>
//!                 <UnifiedFieldAppBar app_name="My App".to_string()>
//!                     <ShellAuthMenu slot:auth_menu>
//!                         <HostAuthMenu />
//!                     </ShellAuthMenu>
//!                 </UnifiedFieldAppBar>
//!             </ShellAppBar>
//!             {children()}
//!         </UnifiedFieldShellLayout>
//!     }
//! }
//! ```
//!
//! On success the shell renders [`UnifiedFieldAppBar`] chrome with your
//! `AppBarUserMenu` in the auth slot. Omit [`ShellAuthMenu`] when the host does
//! not call [`provide_shell_auth_menu`]; [`HostAuthMenu`] then renders nothing.
//! Link `uf-notifications` (and keep `offering-notifications` / `full`) for the
//! default bell in [`HostNotificationBell`], or override with
//! [`provide_shell_notification_bell`].
//!
//! ## Search source picker
//!
//! [`SearchSourcePicker`] offers a multi-source combobox for principal and resource
//! pickers. Parents register search providers once (macros / Quark on SSR), then fill
//! `options` from a server fn that calls `SearchSourceRegistry::query_many`.
//!
//! **Prerequisites:** `uf-search-core` providers registered; `ssr` for server fns.
//!
//! ```rust,ignore
//! use leptos::prelude::*;
//! use uf_integrations::SearchSourcePicker;
//! use uf_search_core::{SearchSourceItem, SearchSourceKey};
//!
//! #[component]
//! fn UserPicker() -> impl IntoView {
//!     let sources = Signal::derive(|| vec![SearchSourceKey::new("users", "Users")]);
//!     let options = RwSignal::new(Vec::<SearchSourceItem>::new());
//!     view! {
//!         <SearchSourcePicker
//!             search_sources=sources
//!             options=options.into()
//!             placeholder="Search users…"
//!             on_search=Callback::new(move |(keys, q)| {
//!                 // server fn → SearchSourceRegistry::query_many → options.set(rows)
//!                 let _ = (keys, q);
//!             })
//!             on_select=Callback::new(move |item| { let _ = item; })
//!         />
//!     }
//! }
//! ```
//!
//! Typing in the picker fires `on_search`; updating `options` renders grouped rows in
//! the combobox. See [`SearchSourcePicker`] `# Examples` for the full callback shape.
//!
//! ## Workspace search
//!
//! [`WorkspaceSearch`] and [`WorkspaceSearchMobileTrigger`] query the per-user workspace
//! content index maintained by `uf_product::workspace_search`. Use them in the app bar
//! (not in standalone pickers—that is [`SearchSourcePicker`]).
//!
//! **Prerequisites:** signed-in session from `uf-product`; indexed content via
//! `SideEffect` writers in source models.
//!
//! ```rust,ignore
//! use leptos::prelude::*;
//! use uf_integrations::{WorkspaceSearch, WorkspaceSearchMobileTrigger};
//!
//! #[component]
//! fn AppBarSearch() -> impl IntoView {
//!     view! {
//!         <WorkspaceSearch />
//!         <WorkspaceSearchMobileTrigger />
//!     }
//! }
//! ```
//!
//! When a signed-in user types a query, the desktop combobox and mobile dialog list
//! workspace index hits and navigate on selection.
//!
//! ## Feature flags
//!
//! | Feature | Effect |
//! |---------|--------|
//! | `default` | Enables `full` (all optional app-bar offerings). |
//! | `offering-help` | Help tour widget in the default utilities pack. |
//! | `offering-apps` | Marker: host should link `uf-apps` so its Apps launcher appears. |
//! | `offering-appearance` | Appearance control in the default utilities pack. |
//! | `offering-notifications` | Marker: host should link `uf-notifications` so inventory fills [`HostNotificationBell`]. |
//! | `full` | All four offering flags above. |
//! | `ssr` | Server fns for workspace search and search-source registry on SSR builds. |
//! | `hydrate` | Client hydration for shell components and search UI. |
//! | `permissions` | Placeholder until Gauge is git-standalone; gates stay fail-closed. |
//!
//! ## Examples
//!
//! Start with `UnifiedFieldShellLayout` + `UnifiedFieldAppBar` + host auth menu +
//! notification bell in [Getting started](#getting-started). Combobox keys and `query_many`
//! options on [`SearchSourcePicker`] item docs. Layout, app bar, search, coming-soon, and
//! 404 on one tree: workspace `examples/shell-chrome-host`.
//!
//! ```bash
//! cargo check -p shell-chrome-host --features ssr
//! ```
//!
//! Lib doctests are disabled (`[lib] doctest = false`); use workspace hosts such as
//! `examples/shell-chrome-host` for compile-checked composition.
//!
//! ## Where to look next
//!
//! - [`UnifiedFieldShellLayout`] — top-level shell composition.
//! - [`UnifiedFieldAppBar`] — the app bar itself.
//! - [`SearchSourcePicker`] — multi-source search combobox.
//! - `uf-product` — session, `RequireAuthenticated`, appearance preferences.
//! - `uf-help` / `uf-apps` / `uf-appearance` — optional app-bar offerings.
//! - `uf-search-core` — `SearchSourceKey` / provider registry.
//! - `examples/shell-chrome-host` — runnable shell chrome composition.

// Leptos `#[component]` / slot expansions still leave many props undocumented.
// Document facades at the crate root and on entry components; keep this allow until
// prop-level docs are ratcheted per component.
#![allow(missing_docs)]
mod app_bar;
mod coming_soon_page;
mod host_auth_menu;
mod host_notification_bell;
mod not_found_page;
mod search_source_picker;
mod shell_layout;
mod workspace_search;

pub use app_bar::{
    AppBarSearchSlot, AppBarTrailingSlot, AppBarUtilities, BreadcrumbLink, DefaultAppBarUtilities,
    ShellAuthMenu, UnifiedFieldAppBar,
};
pub use coming_soon_page::{coming_soon_fill_for_path, UnifiedFieldComingSoonPage};
pub use host_auth_menu::{provide_shell_auth_menu, HostAuthMenu, ShellAuthMenuFactory};
pub use host_notification_bell::{
    collect_shell_notification_bell, provide_shell_notification_bell,
    register_shell_notification_bell, HostNotificationBell, ShellNotificationBellContribution,
    ShellNotificationBellFactory,
};
pub use not_found_page::UnifiedFieldNotFoundPage;
pub use search_source_picker::SearchSourcePicker;
pub use shell_layout::{ShellAppBar, ShellLeftNav, ShellSidebarToggle, UnifiedFieldShellLayout};
pub use workspace_search::{WorkspaceSearch, WorkspaceSearchDialog, WorkspaceSearchMobileTrigger};
