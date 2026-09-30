//! Help spotlight fixture with anchors that are absent, hidden, or below the fold.
#![allow(missing_docs)]

use leptos::prelude::*;
use uf_product::components::{Body1, Title3};
use uf_product::primitives::{Button, Flex, FlexGap};

/// `/help-fixture/deferred`: one anchor mounts on click, one is `display: none`
/// until revealed, and one sits under a tall spacer.
#[component]
pub fn HelpFixtureDeferredPage() -> impl IntoView {
    let mounted = RwSignal::new(false);
    let css_shown = RwSignal::new(false);

    view! {
        <main data-testid="help-fixture-deferred" style="padding: 24px; max-width: 720px;">
            <Flex vertical=true gap=FlexGap::Medium full_width=true>
                <Title3>"Deferred help anchors"</Title3>
                <Flex gap=FlexGap::Small>
                    <Button
                        attr:data-testid="help-fixture-reveal-mount"
                        on_click=Callback::new(move |_| mounted.set(true))
                    >
                        "Mount panel"
                    </Button>
                    <Button
                        attr:data-testid="help-fixture-reveal-css"
                        on_click=Callback::new(move |_| css_shown.set(true))
                    >
                        "Show hidden"
                    </Button>
                </Flex>
                <Show when=move || mounted.get()>
                    <div id="help-fixture-mounted" data-testid="help-fixture-mounted">
                        <Body1>"Mounted panel"</Body1>
                    </div>
                </Show>
                <div
                    id="help-fixture-css-hidden"
                    data-testid="help-fixture-css-hidden"
                    style:display=move || if css_shown.get() { "block" } else { "none" }
                >
                    <Body1>"Hidden panel"</Body1>
                </div>
                <div style="min-height: 1600px;"></div>
                <div id="help-fixture-below-fold" data-testid="help-fixture-below-fold">
                    <Body1>"Below the fold"</Body1>
                </div>
            </Flex>
        </main>
    }
}
