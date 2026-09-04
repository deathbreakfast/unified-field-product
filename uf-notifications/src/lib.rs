#![recursion_limit = "256"]
//! Notification inbox UI for product apps: bell, dropdown preview, and full inbox page.
//!
//! Leptos components and routes for unread badges, notification lists, search, and
//! read-state actions. Valence persistence and `#[server]` implementations live in
//! the domain crates `uf-notifications-core` and `uf-notifications-api`; this crate
//! registers the `notifications` Orbital app and mounts UI into the product shell.
//!
//! # Features
//!
//! - **Notification bell** — App-bar bell with unread badge and dropdown preview wired to
//!   live server functions. [Get started](#mount-shell-bell)
//! - **Inbox routes** — Nest auth-gated `/notifications` routes so signed-in users reach
//!   the full inbox page. [Get started](#mount-inbox-routes)
//! - **Lazy inbox route** — Load the inbox leaf as a separate WASM chunk for
//!   `cargo leptos --split` hosts. [Get started](#lazy-routes)
//!
//! # Getting started
//!
//! Nest [`NotificationsRoutes`] under your router. Enable `ssr` on this crate and
//! `uf-notifications-api` so badge resources call live server functions. With
//! `uf-integrations` `offering-notifications` (in `full`) and this crate linked,
//! [`HostNotificationBell`](uf_integrations::HostNotificationBell) picks up the
//! inventory contribution — no `provide_shell_notification_bell` required.
//!
//! ## Mount shell bell
//!
//! [`NotificationBell`] renders the app-bar dropdown with unread count and preview rows.
//! Register it once at host boot — through inventory (`ensure_notification_bell_linked`
//! at `App()` root, or via [`NotificationsRoutes`] mount) or
//! [`provide_shell_notification_bell`](uf_integrations::provide_shell_notification_bell)
//! for a host-specific factory — so `UnifiedFieldAppBar` can render the slot before
//! routed pages mount. Prefer inventory when the stock bell is enough.
//!
//! Prerequisites: `ssr` on `uf-notifications` and `uf-notifications-api`, signed-in session
//! context from `uf-product`, and `/ws/notifications` mounted on the Axum router for Photon
//! badge refresh via [`server::subscribe_get_unread_count`].
//!
//! ```rust,ignore
//! use leptos::prelude::*;
//! use uf_integrations::provide_shell_notification_bell;
//! use uf_notifications::{NotificationBell, NOTIFICATIONS_ROUTE_PATH};
//!
//! // At App() root — host-specific override of the inventory bell:
//! provide_shell_notification_bell(|| view! { <NotificationBell /> }.into_any());
//! assert_eq!(NOTIFICATIONS_ROUTE_PATH, "/notifications");
//! ```
//!
//! The bell wires [`server::get_unread_count`] and preview paging internally; hosts do not
//! call those server functions directly for chrome. Next: [mount inbox routes](#mount-inbox-routes)
//! for the full page, or the `uf-notifications-api` live unread badge guide for subscription
//! details.
//!
//! ## Mount inbox routes
//!
//! [`NotificationsRoutes`] registers auth-gated `/notifications` routes and exposes the
//! inbox leaf (eager or lazy depending on `lazy-routes`). During host startup, nest it
//! inside your host `<Routes>` before pages render so signed-in users reach
//! [`NotificationsInboxPage`].
//!
//! Prerequisites: same `ssr` session context as [mount shell bell](#mount-shell-bell);
//! [`NotificationsAuthGuardRouteView`] wraps the leaf with
//! `uf_product::routes::RequireAuthenticated`.
//!
//! ```rust,ignore
//! use leptos::prelude::*;
//! use leptos_router::components::Routes;
//! use uf_notifications::{
//!     ensure_help_steps_linked, NotificationsRoutes, NOTIFICATIONS_APP_ID,
//!     NOTIFICATIONS_ROUTE_PATH,
//! };
//!
//! assert_eq!(NOTIFICATIONS_ROUTE_PATH, "/notifications");
//! assert_eq!(NOTIFICATIONS_APP_ID, "notifications");
//! ensure_help_steps_linked();
//!
//! #[component]
//! fn AppRoutes() -> impl IntoView {
//!     view! {
//!         <Routes fallback=|| "not found">
//!             <NotificationsRoutes />
//!         </Routes>
//!     }
//! }
//! ```
//!
//! [`ensure_help_steps_linked`] runs when routes mount so Help spotlight inventory includes
//! inbox tour steps. Next: [lazy routes](#lazy-routes) when using WASM code splitting.
//!
//! ## Lazy routes
//!
//! With `lazy-routes` (the default), the inbox leaf loads through
//! [`NotificationsInboxRoute`] as a separate WASM chunk while
//! [`NotificationsAuthGuardRouteView`] stays eager for auth gating. Call
//! [`prefetch_family`] after sign-in if you want to hide first-paint latency on `/notifications`.
//!
//! Prerequisites: `lazy-routes` enabled (default), `ssr`/`hydrate` for badge subscription,
//! and [mount inbox routes](#mount-inbox-routes) already nested under the host router.
//!
//! ```rust,ignore
//! # async fn demo() {
//! use leptos_router::{Lazy, LazyRoute};
//! use uf_notifications::{prefetch_family, NotificationsInboxRoute, NOTIFICATIONS_ROUTE_PATH};
//!
//! assert_eq!(NOTIFICATIONS_ROUTE_PATH, "/notifications");
//! assert_eq!(NotificationsInboxRoute::data(), NotificationsInboxRoute);
//! let inbox_lazy = Lazy::<NotificationsInboxRoute>::new();
//! prefetch_family().await;
//! // Preload finished; nest `inbox_lazy` under NotificationsRoutes for cargo-leptos --split.
//! let _keep = inbox_lazy;
//! # }
//! ```
//!
//! Disable `lazy-routes` for single-bundle hosts; the inbox page then bundles inline.
//! Either path still needs `ssr` or `hydrate` on the API crate for a working
//! [`server::subscribe_get_unread_count`] badge subscription.
//!
//! ## Examples
//!
//! Start with [Mount shell bell](#mount-shell-bell) and [mount inbox routes](#mount-inbox-routes).
//! Domain workspace `examples/notifications-mount-host` exercises inventory id and the
//! `/notifications` auth gate (Axum oneshot). Product shell chrome:
//! `examples/shell-chrome-host`.
//!
//! ## Where to look next
//!
//! - [`mod@server`] — client-callable server functions re-exported from `uf-notifications-api`.
//! - [`NotificationsInboxPage`] — full inbox with search, filter, and pagination.
//! - `uf-notifications-core` — persist and publish path for backend callers.
//!
//! # Feature flags
//!
//! | Feature | Default | Purpose |
//! |---------|---------|---------|
//! | `lazy-routes` | yes | Lazy-load inbox WASM chunk via [`NotificationsInboxRoute`] |
//! | `ssr` | no | Server functions, Valence, and Photon synced unread count |
//! | `hydrate` | no | Client hydration; enables real [`server::subscribe_get_unread_count`] |
//! | `dev-tools` | no | Re-export `create_test_notification` for local UI and e2e |

#![allow(missing_docs)]

use leptos::prelude::*;
#[cfg(feature = "lazy-routes")]
use leptos_router::Lazy;
use leptos_router::{
    components::{ParentRoute, Route},
    path,
};
use uf_integrations::{register_shell_notification_bell, ShellNotificationBellContribution};
use uf_product_macros::uf_app;

mod components;
mod help_steps;
mod layout;
mod lazy_routes;
mod pages;
mod safe_url;
/// Client-callable notification server functions, re-exported from `uf-notifications-api`.
pub mod server;
mod surface_layout;

pub use components::NotificationBell;

pub use help_steps::ensure_help_steps_linked;
pub use layout::NotificationsLayout;
pub use lazy_routes::NotificationsAuthGuardRouteView;
#[cfg(feature = "lazy-routes")]
pub use lazy_routes::{prefetch_family, NotificationsInboxRoute};
pub use pages::NotificationsInboxPage;
pub use surface_layout::{BELL_DROPDOWN_MAX_WIDTH, BELL_DROPDOWN_MIN_WIDTH, INBOX_MIN_WIDTH};

/// `uf_app!` id for the notifications product (`notifications-mount-host` mirrors this).
pub const NOTIFICATIONS_APP_ID: &str = "notifications";

/// Inbox route path from `uf_app!` (`/notifications`).
pub const NOTIFICATIONS_ROUTE_PATH: &str = "/notifications";

fn render_shell_notification_bell() -> AnyView {
    view! { <NotificationBell /> }.into_any()
}

inventory::submit! {
    ShellNotificationBellContribution::new(render_shell_notification_bell)
}

/// Force-link the shell notification bell inventory submission.
///
/// Call from the host App root when routes may not mount this crate early enough
/// for `HostNotificationBell` to see the contribution. [`NotificationsRoutes`]
/// also calls this on mount.
pub fn ensure_notification_bell_linked() {
    register_shell_notification_bell();
}

uf_app! {
    name: "Notifications",
    id: "notifications",
    description: "User notification inbox",
    icon: "🔔",
    version: "0.1.0",
    routes: NotificationsRoutes,
    route_path: "/notifications",
}

/// Notifications app routes: an auth-gated inbox page at `/notifications`.
#[allow(missing_docs)]
#[orbital_macros::orbital_routes_extract]
#[component(transparent)]
pub fn NotificationsRoutes() -> impl leptos_router::MatchNestedRoutes + Clone {
    crate::help_steps::ensure_help_steps_linked();
    ensure_notification_bell_linked();
    #[cfg(feature = "lazy-routes")]
    {
        view! {
            <ParentRoute path=path!("notifications") view=NotificationsAuthGuardRouteView>
                <Route path=path!("") view={Lazy::<NotificationsInboxRoute>::new()} />
            </ParentRoute>
        }
        .into_inner()
    }
    #[cfg(not(feature = "lazy-routes"))]
    {
        view! {
            <ParentRoute path=path!("notifications") view=NotificationsAuthGuardRouteView>
                <Route path=path!("") view=NotificationsInboxPage />
            </ParentRoute>
        }
        .into_inner()
    }
}
