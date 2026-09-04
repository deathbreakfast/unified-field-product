use leptos::prelude::*;
use turf::inline_style_sheet_values;
use uf_product::components::{
    AutoGrid, Caption1, ContentContainer, EmptyState, FlexGap, OrbitalInfiniteScroll,
    OrbitalInfiniteScrollEmptyView, OrbitalInfiniteScrollEndView, EMPTYSTATE_SAD_DOG_ILLUSTRATION,
};
use uf_product::primitives::{Flex, FlexAlign, FlexJustify};

use crate::components::{AppCard, AppSearchInput};
use crate::server::{get_apps_page, APPS_PAGE_SIZE};

/// Apps directory index: searchable, paginated grid of every registered app.
///
/// Wires the index search input to [`crate::server::get_apps_page`] through
/// [`uf_product::components::OrbitalInfiniteScroll`]. Re-mounts the scroll
/// host when the query changes so offset resets. Empty results use
/// [`uf_product::components::EmptyState`].
#[component]
pub fn AppsIndexPage() -> impl IntoView {
    crate::help_steps::ensure_help_steps_linked();
    let query = RwSignal::new(String::new());

    // Fill the shell main scrollport so the infinite-scroll viewport can use height: 100%
    // instead of a magic `100dvh - Npx` that leaves a bottom gap.
    let (style_sheet, class_names) = inline_style_sheet_values! {
        .Page {
            box-sizing: border-box;
            height: calc(
                100dvh
                - var(--orbital-layout-header-inset, 48px)
                - 2 * var(--orb-space-block-lg, 16px)
            );
            max-height: calc(
                100dvh
                - var(--orbital-layout-header-inset, 48px)
                - 2 * var(--orb-space-block-lg, 16px)
            );
        }

        .ScrollHost {
            flex: 1 1 auto;
            min-height: 0;
        }

        .ScrollHost > [data-testid="orbital-infinite-scroll"] {
            flex: 1 1 auto;
            height: 100%;
            max-height: 100%;
        }
    };

    view! {
        <style>{style_sheet}</style>
        <ContentContainer data_testid="apps-index-page" class=class_names.page>
            <Flex vertical=true fill=true full_width=true gap=FlexGap::Medium>
                <AppSearchInput value=query />

                // Re-mount the infinite scroll when the search query changes.
                {move || {
                    let current_query = query.get();
                    let q = if current_query.trim().is_empty() {
                        None
                    } else {
                        Some(current_query)
                    };
                    let fetch_apps = move |offset: u32, limit: u32| {
                        get_apps_page(offset, limit, q.clone())
                    };
                    let scroll_host = class_names.scroll_host.to_string();

                    view! {
                        <Flex vertical=true fill=true full_width=true class=scroll_host>
                            <OrbitalInfiniteScroll
                                page_size=APPS_PAGE_SIZE
                                fetch=fetch_apps
                                max_height="100%"
                                let:items
                            >
                                <OrbitalInfiniteScrollEmptyView slot>
                                    <EmptyState
                                        message="No apps registered"
                                        description="Installed apps will appear here."
                                        illustration_src=EMPTYSTATE_SAD_DOG_ILLUSTRATION
                                        illustration_alt="No apps"
                                    />
                                </OrbitalInfiniteScrollEmptyView>
                                <OrbitalInfiniteScrollEndView slot>
                                    <Flex justify=FlexJustify::Center align=FlexAlign::Center full_width=true>
                                        <Caption1>"All apps loaded"</Caption1>
                                    </Flex>
                                </OrbitalInfiniteScrollEndView>
                                <AutoGrid min="270px">
                                    <For
                                        each=move || {
                                            items
                                                .get()
                                                .into_iter()
                                                .enumerate()
                                                .collect::<Vec<_>>()
                                        }
                                        key=|(_i, app)| app.slug.clone()
                                        let:item
                                    >
                                        {
                                            let (index, app) = item;
                                            view! { <AppCard app=app first=index == 0 /> }
                                        }
                                    </For>
                                </AutoGrid>
                            </OrbitalInfiniteScroll>
                        </Flex>
                    }
                }}
            </Flex>
        </ContentContainer>
    }
}
