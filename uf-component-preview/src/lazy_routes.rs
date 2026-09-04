//! Lazy-loaded route views for WASM code-splitting (`cargo leptos --split`).

use leptos::prelude::*;
use leptos_router::{lazy_route, LazyRoute};

use crate::pages::{ComponentPreview, IntroductionPage};
use crate::preview::PreviewSlugPage;

/// Prefetch the orbital dev family WASM chunk (leaf pages share split modules).
pub async fn prefetch_family() {
    IntroductionPageRoute::preload().await;
}

/// Lazy `/orbital` and `/orbital/components` introduction page.
#[derive(Clone, Copy, Debug, Default)]
pub struct IntroductionPageRoute;

#[lazy_route]
impl LazyRoute for IntroductionPageRoute {
    fn data() -> Self {
        Self
    }

    fn view(_this: Self) -> AnyView {
        view! { <IntroductionPage /> }.into_any()
    }
}

/// Lazy `/orbital/shell` component preview page.
#[derive(Clone, Copy, Debug, Default)]
pub struct ComponentPreviewRoute;

#[lazy_route]
impl LazyRoute for ComponentPreviewRoute {
    fn data() -> Self {
        Self
    }

    fn view(_this: Self) -> AnyView {
        view! { <ComponentPreview /> }.into_any()
    }
}

/// Lazy `/orbital/*slug` registry-driven preview page.
#[derive(Clone, Copy, Debug, Default)]
pub struct PreviewSlugPageRoute;

#[lazy_route]
impl LazyRoute for PreviewSlugPageRoute {
    fn data() -> Self {
        Self
    }

    fn view(_this: Self) -> AnyView {
        view! { <PreviewSlugPage /> }.into_any()
    }
}
