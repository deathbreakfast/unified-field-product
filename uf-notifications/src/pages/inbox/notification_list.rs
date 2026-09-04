use std::collections::HashMap;

use leptos::prelude::*;
use uuid::Uuid;

use uf_product::components::{
    Caption1, EmptyState, OrbitalInfiniteScroll, OrbitalInfiniteScrollEmptyView,
    OrbitalInfiniteScrollEndView, SpacingSize, EMPTYSTATE_SAD_DOG_ILLUSTRATION,
};
use uf_product::primitives::Flex;

use super::NotificationItem;
use crate::server::{get_notifications_page, NotificationReadFilter};

/// Number of notifications per page.
const NOTIFICATIONS_PAGE_SIZE: u32 = 20;

/// Infinite-scrolling, filterable/searchable list of notification rows.
///
/// Re-mounts the underlying [`OrbitalInfiniteScroll`] whenever `search_query` or `filter`
/// changes, so pagination restarts cleanly for the new query.
#[component]
pub fn NotificationList(
    /// Two-way signal holding the filter value to apply.
    filter: RwSignal<NotificationReadFilter>,
    /// Reactive signal for the search query.
    search_query: ReadSignal<String>,
    /// Two-way signal holding the read overrides.
    read_overrides: RwSignal<HashMap<Uuid, bool>>,
    /// Bumped after bulk ops (mark-all) so the list remounts and refetches.
    reload: RwSignal<u64>,
    /// Navigation callback used to change routes.
    navigate: impl Fn(&str, leptos_router::NavigateOptions) + Clone + Send + Sync + 'static,
) -> impl IntoView {
    // Re-mount the infinite scroll when search, filter, or reload changes.
    move || {
        let _ = reload.get();
        let current_query = search_query.get();
        let rf = filter.get();

        let q = if current_query.trim().is_empty() {
            None
        } else {
            Some(current_query)
        };

        let fetch_notifications =
            move |offset: u32, limit: u32| get_notifications_page(offset, limit, q.clone(), rf);

        let navigate = navigate.clone();
        view! {
            <OrbitalInfiniteScroll
                page_size=NOTIFICATIONS_PAGE_SIZE
                fetch=fetch_notifications
                max_height="600px"
                let:items
            >
                <OrbitalInfiniteScrollEmptyView slot>
                    <EmptyState
                        message="No notifications"
                        description="You're all caught up. New notifications will appear here."
                        illustration_src=EMPTYSTATE_SAD_DOG_ILLUSTRATION
                        illustration_alt="No notifications"
                    />
                </OrbitalInfiniteScrollEmptyView>
                <OrbitalInfiniteScrollEndView slot>
                    <Caption1>"End of notifications"</Caption1>
                </OrbitalInfiniteScrollEndView>
                {
                    let navigate = navigate.clone();
                    view! {
                        <Flex vertical=true gap=SpacingSize::Size120.flex_gap()>
                            <For
                                each=move || items.get()
                                key=|n| n.notification_id
                                let:notification
                            >
                                {
                                    let navigate = navigate.clone();
                                    view! {
                                        <NotificationItem
                                            notification=notification
                                            read_overrides=read_overrides
                                            navigate=navigate
                                        />
                                    }
                                }
                            </For>
                        </Flex>
                    }
                }
            </OrbitalInfiniteScroll>
        }
    }
}
