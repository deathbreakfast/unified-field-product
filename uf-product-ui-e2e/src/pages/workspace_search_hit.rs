//! Destination route for workspace search e2e navigation.
#![allow(missing_docs)]

use leptos::prelude::*;
use uf_product::components::{Body1, Title3};
use uf_product::primitives::{Flex, FlexGap};

/// Stable page for `IndexedDemoItem.link` / Playwright select → navigate.
#[component]
pub fn WorkspaceSearchHitPage() -> impl IntoView {
    view! {
        <main data-testid="workspace-search-destination" style="padding: 24px; max-width: 720px;">
            <Flex vertical=true gap=FlexGap::Medium full_width=true>
                <Title3>"Workspace search hit"</Title3>
                <Body1>"Opened from AppBar workspace content search."</Body1>
            </Flex>
        </main>
    }
}
