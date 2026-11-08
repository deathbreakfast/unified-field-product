//! Home page: SearchSourcePicker + default utilities chrome.
#![allow(missing_docs)]

use leptos::prelude::*;
use uf_integrations::SearchSourcePicker;
use uf_product::components::{Body1, Title3};
use uf_product::primitives::{Flex, FlexGap};
use uf_search_core::{SearchSourceItem, SearchSourceKey};

fn server_match_item() -> SearchSourceItem {
    SearchSourceItem {
        source_id: "beacons".into(),
        id: "gamma".into(),
        title: "Beacon Gamma".into(),
        description: Some("ops@example.com".into()),
        kind: "beacon".into(),
        image_url: None,
    }
}

const PIXEL: &str = "data:image/gif;base64,R0lGODlhAQABAAAAACwAAAAAAQABAAA=";

fn person(id: &str, name: &str, image_url: Option<&str>) -> SearchSourceItem {
    SearchSourceItem {
        source_id: "people".into(),
        id: id.into(),
        title: name.into(),
        description: Some("hidden-by-custom-row".into()),
        kind: "person".into(),
        image_url: image_url.map(Into::into),
    }
}

/// Row drawn by `option_view`: the photo when the source supplied one, then the name.
fn person_row(item: SearchSourceItem) -> AnyView {
    view! {
        <Flex gap=FlexGap::Small attr:data-testid="custom-row">
            {item.image_url.map(|url| view! {
                // Orbital escape: the demo draws a bare photo; no Avatar fixture URL here.
                <img data-testid="custom-row-image" src=url alt="" width="24" height="24" />
            })}
            <Body1>{item.title}</Body1>
        </Flex>
    }
    .into_any()
}

#[component]
pub fn HomePage() -> impl IntoView {
    let sources = Signal::derive(|| vec![SearchSourceKey::new("beacons", "Beacons")]);
    let options = RwSignal::new(vec![
        SearchSourceItem {
            source_id: "beacons".into(),
            id: "alpha".into(),
            title: "Beacon Alpha".into(),
            description: Some("Stub option for the e2e host".into()),
            kind: "beacon".into(),
            image_url: None,
        },
        SearchSourceItem {
            source_id: "beacons".into(),
            id: "beta".into(),
            title: "Beacon Beta".into(),
            description: None,
            kind: "beacon".into(),
            image_url: None,
        },
    ]);
    let selected = RwSignal::new(None::<String>);
    let people_sources = Signal::derive(|| vec![SearchSourceKey::new("people", "People")]);
    let people: Signal<Vec<SearchSourceItem>> = Signal::derive(|| {
        vec![
            person("ada", "Ada Lovelace", Some(PIXEL)),
            person("bo", "Bo Plain", None),
        ]
    });
    let server_options = RwSignal::new(Vec::<SearchSourceItem>::new());
    let server_sig: Signal<Vec<SearchSourceItem>> = server_options.into();
    let local_options = RwSignal::new(Vec::<SearchSourceItem>::new());
    let local_sig: Signal<Vec<SearchSourceItem>> = local_options.into();
    let options_sig: Signal<Vec<SearchSourceItem>> = options.into();

    view! {
        <main data-testid="shell-chrome-home" style="padding: 24px; max-width: 720px;">
            <Flex vertical=true gap=FlexGap::Medium full_width=true>
                <Title3>"Product UI e2e"</Title3>
                <Body1>
                    "Consumer-wired shell with default app-bar utilities and search picker."
                </Body1>
                <div data-testid="shell-chrome-search">
                    <SearchSourcePicker
                        search_sources=sources
                        options=options_sig
                        placeholder="Search beacons…"
                        on_search=Callback::new(move |(_keys, q): (Vec<SearchSourceKey>, String)| {
                            if q.trim().is_empty() {
                                options.set(vec![
                                    SearchSourceItem {
                                        source_id: "beacons".into(),
                                        id: "alpha".into(),
                                        title: "Beacon Alpha".into(),
                                        description: Some("Stub option for the e2e host".into()),
                                        kind: "beacon".into(),
                                        image_url: None,
                                    },
                                    SearchSourceItem {
                                        source_id: "beacons".into(),
                                        id: "beta".into(),
                                        title: "Beacon Beta".into(),
                                        description: None,
                                        kind: "beacon".into(),
                                        image_url: None,
                                    },
                                ]);
                            } else if q.to_lowercase().contains("zz-no-match") {
                                options.set(Vec::new());
                            } else {
                                let needle = q.to_lowercase();
                                options.update(|items| {
                                    items.retain(|i| i.title.to_lowercase().contains(&needle));
                                });
                            }
                        })
                        on_select=Callback::new(move |item: SearchSourceItem| {
                            selected.set(Some(item.title));
                        })
                    />
                </div>
                // Stands in for a server search that matches on a field other than the
                // title: any non-empty query returns the same row.
                <div data-testid="server-match-search">
                    <SearchSourcePicker
                        search_sources=sources
                        options=server_sig
                        placeholder="Search by email…"
                        on_search=Callback::new(move |(_keys, q): (Vec<SearchSourceKey>, String)| {
                            server_options.set(if q.trim().is_empty() {
                                Vec::new()
                            } else {
                                vec![server_match_item()]
                            });
                        })
                    />
                </div>
                <div data-testid="server-match-local-filter-search">
                    <SearchSourcePicker
                        search_sources=sources
                        options=local_sig
                        local_filter=true
                        placeholder="Search by email (local filter)…"
                        on_search=Callback::new(move |(_keys, q): (Vec<SearchSourceKey>, String)| {
                            local_options.set(if q.trim().is_empty() {
                                Vec::new()
                            } else {
                                vec![server_match_item()]
                            });
                        })
                    />
                </div>
                <div data-testid="custom-row-search">
                    <SearchSourcePicker
                        search_sources=people_sources
                        options=people
                        placeholder="Search people…"
                        option_view=Callback::new(person_row)
                        value_view=Callback::new(|item: SearchSourceItem| {
                            view! { <span data-testid="custom-value">{item.title}</span> }.into_any()
                        })
                    />
                </div>
                <p data-testid="search-selected">
                    {move || selected.get().unwrap_or_default()}
                </p>
            </Flex>
        </main>
    }
}
