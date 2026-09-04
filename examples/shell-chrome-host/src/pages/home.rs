//! Home page: SearchSourcePicker with stub options inside the product shell.
#![allow(missing_docs)]

use leptos::prelude::*;
use uf_integrations::SearchSourcePicker;
use uf_product::components::{Body1, Title3};
use uf_product::primitives::{Flex, FlexGap};
use uf_search_core::{SearchSourceItem, SearchSourceKey};

#[component]
pub fn HomePage() -> impl IntoView {
    let sources = Signal::derive(|| vec![SearchSourceKey::new("beacons", "Beacons")]);
    let options = RwSignal::new(vec![
        SearchSourceItem {
            source_id: "beacons".into(),
            id: "alpha".into(),
            title: "Beacon Alpha".into(),
            description: Some("Stub option for the shell host".into()),
            kind: "beacon".into(),
        },
        SearchSourceItem {
            source_id: "beacons".into(),
            id: "beta".into(),
            title: "Beacon Beta".into(),
            description: None,
            kind: "beacon".into(),
        },
    ]);
    let options_sig: Signal<Vec<SearchSourceItem>> = options.into();

    view! {
        <main data-testid="shell-chrome-home" style="padding: 24px; max-width: 720px;">
            <Flex vertical=true gap=FlexGap::Medium full_width=true>
                <Title3>"Shell chrome"</Title3>
                <Body1>
                    "This host composes UnifiedFieldShellLayout, UnifiedFieldAppBar, SearchSourcePicker, and links to coming-soon / 404 pages. Apps and Welcome are mounted as real uf_app! routes."
                </Body1>
                <div data-testid="shell-chrome-search">
                    <SearchSourcePicker
                        search_sources=sources
                        options=options_sig
                        placeholder="Search beacons…"
                        on_search=Callback::new(move |(_keys, _q)| {})
                        on_select=Callback::new(move |_item| {})
                    />
                </div>
            </Flex>
        </main>
    }
}
