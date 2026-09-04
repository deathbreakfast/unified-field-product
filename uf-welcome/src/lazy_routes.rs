//! Lazy-loaded route views for WASM code-splitting (`cargo leptos --split`).

use leptos::prelude::*;
use leptos_router::{lazy_route, LazyRoute};

use crate::welcome::{WelcomeAdminPage, WelcomePage};

/// Prefetch the welcome family WASM chunk (leaf pages share split modules).
pub async fn prefetch_family() {
    WelcomePageRoute::preload().await;
}

/// Lazy `/welcome` page.
#[derive(Clone, Copy, Debug, Default)]
pub struct WelcomePageRoute;

#[lazy_route]
impl LazyRoute for WelcomePageRoute {
    fn data() -> Self {
        Self
    }

    fn view(_this: Self) -> AnyView {
        view! { <WelcomePage /> }.into_any()
    }
}

/// Lazy `/welcome/admin` page.
#[derive(Clone, Copy, Debug, Default)]
pub struct WelcomeAdminPageRoute;

#[lazy_route]
impl LazyRoute for WelcomeAdminPageRoute {
    fn data() -> Self {
        Self
    }

    fn view(_this: Self) -> AnyView {
        view! { <WelcomeAdminPage /> }.into_any()
    }
}
