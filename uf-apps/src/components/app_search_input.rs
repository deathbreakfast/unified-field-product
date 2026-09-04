use leptos::prelude::*;
use uf_product::components::{SearchBox, SearchBoxAppearance, SearchBoxBind};
use uf_product::primitives::{Flex, FlexJustify};

/// Search bar for the apps directory index page.
#[component]
pub fn AppSearchInput(
    /// Two-way signal holding the current value.
    value: RwSignal<String>,
) -> impl IntoView {
    view! {
                <Flex justify=FlexJustify::Center full_width=true>
            <div id="apps-search-input" data-testid="apps-search-input">
                <SearchBox
                    bind=SearchBoxBind::from(value)
                    appearance=SearchBoxAppearance::with_placeholder("Search apps")
                />
            </div>
        </Flex>
    }
}
