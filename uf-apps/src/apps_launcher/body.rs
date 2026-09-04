//! Shared search field + results body for the apps launcher dialog.

use leptos::prelude::*;
use turf::inline_style_sheet_values;
use uf_product::components::EmptyState;
use uf_product::primitives::{
    Flex, FlexGap, FlexJustify, List, MessageBar, MessageBarIntent, SearchBox, SearchBoxAppearance,
    SearchBoxBind, Spinner,
};

use super::result_row::AppsLauncherResult;
use crate::server::{filter_apps_by_query, get_apps, AppDirectoryItem};

/// Search box, empty states, and filtered app list for the apps launcher.
#[component]
pub fn AppsLauncherBody(
    /// Two-way search query. Empty (whitespace-only) shows the type-to-search prompt.
    query: RwSignal<String>,
    /// Called with a safe `route_path` when the user picks an app.
    on_select: Callback<String>,
) -> impl IntoView {
    let query_trimmed = Memo::new(move |_| query.get().trim().to_string());
    let has_query = Memo::new(move |_| !query_trimmed.get().is_empty());

    // Fetch the full registry once the user types; keep the snapshot for local typeahead.
    let apps_res = Resource::new(
        move || has_query.get(),
        |load| async move {
            if !load {
                return Ok::<Option<Vec<AppDirectoryItem>>, ServerFnError>(None);
            }
            get_apps().await.map(Some)
        },
    );

    // SearchBox is content-sized by default; stretch it to the dialog content column.
    let (style_sheet, class_names) = inline_style_sheet_values! {
        .SearchHost {
            width: 100%;
            box-sizing: border-box;
        }

        .SearchHost > * {
            width: 100%;
            box-sizing: border-box;
        }
    };

    view! {
        <style>{style_sheet}</style>
        <Flex vertical=true gap=FlexGap::Medium full_width=true>
            <div data-testid="apps-launcher-search" class=class_names.search_host>
                <SearchBox
                    bind=SearchBoxBind::from(query)
                    appearance=SearchBoxAppearance::with_placeholder("Search apps")
                />
            </div>

            {move || {
                if !has_query.get() {
                    return view! {
                        <div data-testid="apps-launcher-empty-prompt">
                            <EmptyState
                                message="Type to find an app"
                                description="Results appear as you type."
                                icon=icondata::AiSearchOutlined
                            />
                        </div>
                    }
                    .into_any();
                }

                match apps_res.get() {
                    None => view! {
                        <div data-testid="apps-launcher-loading">
                            <Flex full_width=true justify=FlexJustify::Center>
                                <Spinner />
                            </Flex>
                        </div>
                    }
                    .into_any(),
                    Some(Err(_)) => view! {
                        <MessageBar intent=MessageBarIntent::Error>
                            "Couldn't load apps. Check your connection and try again."
                        </MessageBar>
                    }
                    .into_any(),
                    Some(Ok(None)) => view! {
                        <div data-testid="apps-launcher-empty-prompt">
                            <EmptyState
                                message="Type to find an app"
                                description="Results appear as you type."
                                icon=icondata::AiSearchOutlined
                            />
                        </div>
                    }
                    .into_any(),
                    Some(Ok(Some(apps))) => {
                        let q = query_trimmed.get();
                        let mut filtered = apps;
                        filter_apps_by_query(&mut filtered, Some(&q));
                        if filtered.is_empty() {
                            view! {
                                <div data-testid="apps-launcher-empty-no-match">
                                    <EmptyState
                                        message="No apps match"
                                        description="Try a different name."
                                        icon=icondata::AiSearchOutlined
                                    />
                                </div>
                            }
                            .into_any()
                        } else {
                            view! {
                                <List>
                                    <For
                                        each=move || filtered.clone()
                                        key=|app| app.slug.clone()
                                        let:app
                                    >
                                        <AppsLauncherResult app=app on_select=on_select />
                                    </For>
                                </List>
                            }
                            .into_any()
                        }
                    }
                }
            }}
        </Flex>
    }
}
