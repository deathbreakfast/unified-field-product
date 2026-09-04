//! Lazy-loaded route views for WASM code-splitting (`cargo leptos --split`).

use leptos::prelude::*;
use leptos_router::{lazy_route, LazyRoute};

use crate::pages::{AppDetailPage, AppsIndexPage};

/// Prefetch the apps-directory family WASM chunk (leaf pages share split modules).
pub async fn prefetch_family() {
    AppsIndexRoute::preload().await;
}

/// Lazy `/apps` index.
#[derive(Clone, Copy, Debug, Default)]
pub struct AppsIndexRoute;

#[lazy_route]
impl LazyRoute for AppsIndexRoute {
    fn data() -> Self {
        Self
    }

    fn view(_this: Self) -> AnyView {
        view! { <AppsIndexPage /> }.into_any()
    }
}

/// Lazy `/apps/:app_name` detail.
#[derive(Clone, Copy, Debug, Default)]
pub struct AppDetailRoute;

#[lazy_route]
impl LazyRoute for AppDetailRoute {
    fn data() -> Self {
        Self
    }

    fn view(_this: Self) -> AnyView {
        view! { <AppDetailPage /> }.into_any()
    }
}
