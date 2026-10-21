//! Fixture for the `AdaptiveSheet` Playwright scenarios.
#![allow(missing_docs)]

use leptos::prelude::*;
use uf_product::components::{Body1, Title3};
use uf_product::primitives::{Button, ButtonAppearance, DrawerSize, Flex, FlexGap, Input};
use uf_product::AdaptiveSheet;

/// Three sheets sharing one confirm counter: Full with a caller-owned input,
/// Small, and an info-only sheet with a single action.
#[component]
pub fn AdaptiveSheetPage() -> impl IntoView {
    let full_open = RwSignal::new(false);
    let small_open = RwSignal::new(false);
    let info_open = RwSignal::new(false);
    let draft = RwSignal::new(String::new());
    let confirms = RwSignal::new(0u32);

    view! {
        <main data-testid="adaptive-sheet-fixture" style="padding: 24px; max-width: 720px;">
            <Flex vertical=true gap=FlexGap::Medium full_width=true>
                <Title3>"Adaptive sheet fixture"</Title3>
                <Body1>
                    "Open state: "
                    <span data-testid="demo-sheet-open-state">
                        {move || if full_open.get() { "open" } else { "closed" }}
                    </span>
                    " · Confirms: "
                    <span data-testid="demo-sheet-confirms">{move || confirms.get()}</span>
                </Body1>
                <Flex gap=FlexGap::Small>
                    <Button
                        attr:data-testid="demo-sheet-trigger"
                        on_click=Callback::new(move |_| full_open.set(true))
                    >
                        "Open sheet"
                    </Button>
                    <Button
                        attr:data-testid="demo-sheet-small-trigger"
                        on_click=Callback::new(move |_| small_open.set(true))
                    >
                        "Open small sheet"
                    </Button>
                    <Button
                        attr:data-testid="demo-sheet-info-trigger"
                        on_click=Callback::new(move |_| info_open.set(true))
                    >
                        "Open info sheet"
                    </Button>
                </Flex>
            </Flex>

            <AdaptiveSheet
                open=full_open
                title=|| "Demo sheet"
                testid="demo-sheet"
                actions=move || {
                    view! {
                        <Button
                            attr:data-testid="demo-sheet-cancel"
                            appearance=ButtonAppearance::Secondary
                            on_click=Callback::new(move |_| full_open.set(false))
                        >
                            "Cancel"
                        </Button>
                        <Button
                            attr:data-testid="demo-sheet-confirm"
                            appearance=ButtonAppearance::Primary
                            on_click=Callback::new(move |_| confirms.update(|n| *n += 1))
                        >
                            "Confirm"
                        </Button>
                    }
                }
            >
                <div data-testid="demo-sheet-input">
                    <Input bind=draft />
                </div>
            </AdaptiveSheet>

            <AdaptiveSheet
                open=small_open
                title=|| "Small sheet"
                testid="demo-sheet-small"
                size=DrawerSize::Small
                actions=move || {
                    view! {
                        <Button on_click=Callback::new(move |_| small_open.set(false))>
                            "Close"
                        </Button>
                    }
                }
            >
                <Body1>"A short confirmation."</Body1>
            </AdaptiveSheet>

            <AdaptiveSheet
                open=info_open
                title=|| "Info sheet"
                testid="demo-sheet-info"
                size=DrawerSize::Small
                actions=move || {
                    view! {
                        <Button
                            attr:data-testid="demo-sheet-info-close"
                            on_click=Callback::new(move |_| info_open.set(false))
                        >
                            "Close"
                        </Button>
                    }
                }
            >
                <Body1>"Nothing to decide here."</Body1>
            </AdaptiveSheet>
        </main>
    }
}
