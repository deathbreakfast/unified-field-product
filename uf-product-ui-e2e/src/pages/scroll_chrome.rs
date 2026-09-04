//! Tall scroll fixture for app-bar hide-on-scroll Playwright scenarios.
#![allow(missing_docs)]

use leptos::prelude::*;
use uf_product::components::{Body1, Title3};
use uf_product::primitives::{Flex, FlexGap};

/// Page body tall enough to scroll the shell page ScrollArea on a phone viewport.
#[component]
pub fn ScrollChromePage() -> impl IntoView {
    view! {
        <main data-testid="shell-chrome-scroll-fixture" style="padding: 24px; max-width: 720px;">
            <Flex vertical=true gap=FlexGap::Medium full_width=true>
                <Title3>"Scroll chrome fixture"</Title3>
                <Body1>
                    "Tall content so the product shell page ScrollArea can drive hide-on-scroll."
                </Body1>
                <div style="min-height: 2200px; display: flex; flex-direction: column; gap: 24px;">
                    {(0..40)
                        .map(|i| {
                            view! {
                                <p style="margin: 0;">
                                    {format!("Scroll fixture line {i}")}
                                </p>
                            }
                        })
                        .collect_view()}
                </div>
            </Flex>
        </main>
    }
}
