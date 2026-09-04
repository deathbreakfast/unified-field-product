//! Lazy-loaded route views for WASM code-splitting (`cargo leptos --split`).

use leptos::prelude::*;

use crate::layout::NotificationsLayout;
use uf_product::routes::RequireAuthenticated;

/// Eager auth-gated layout shell for `/notifications/*` ParentRoute.
#[component]
pub fn NotificationsAuthGuardRouteView() -> impl IntoView {
    view! {
        <RequireAuthenticated>
            <NotificationsLayout />
        </RequireAuthenticated>
    }
}

/// Prefetch the notifications family WASM chunk (leaf pages share split modules).
#[cfg(feature = "lazy-routes")]
pub async fn prefetch_family() {
    NotificationsInboxRoute::preload().await;
}

/// Lazy `/notifications` inbox page.
#[cfg(feature = "lazy-routes")]
#[derive(Clone, Copy, Debug, Default)]
pub struct NotificationsInboxRoute;

#[cfg(feature = "lazy-routes")]
use crate::pages::NotificationsInboxPage;
#[cfg(feature = "lazy-routes")]
use leptos_router::{lazy_route, LazyRoute};

#[cfg(feature = "lazy-routes")]
#[lazy_route]
impl LazyRoute for NotificationsInboxRoute {
    fn data() -> Self {
        Self
    }

    fn view(_this: Self) -> AnyView {
        view! { <NotificationsInboxPage /> }.into_any()
    }
}
