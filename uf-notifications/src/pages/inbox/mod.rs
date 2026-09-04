mod notification_item;
mod notification_list;
mod stats_grid;
mod toolbar;

pub use notification_item::NotificationItem;
pub use notification_list::NotificationList;
pub use stats_grid::NotificationStatsGrid;
pub use toolbar::NotificationToolbar;

use std::collections::HashMap;

use leptos::prelude::*;
use leptos::task::spawn_local_scoped;
use leptos_router::hooks::use_navigate;
use uf_product::components::{ContentContainer, SpacingSize, Title3};
use uf_product::primitives::Flex;
use uuid::Uuid;

use crate::server::{mark_all_notifications_read, NotificationReadFilter};
use crate::surface_layout::INBOX_MIN_WIDTH;

/// Full notification inbox: stats grid, search/filter toolbar, and a paginated list
/// with optimistic read-state overrides while bulk operations are in flight.
#[component]
pub fn NotificationsInboxPage() -> impl IntoView {
    let filter = RwSignal::new(NotificationReadFilter::All);
    let search_query = RwSignal::new(String::new());
    let read_overrides = RwSignal::new(HashMap::<Uuid, bool>::new());
    let navigate = use_navigate();

    // Trigger bumped after bulk operations so stats and the list refetch.
    let stats_trigger = RwSignal::new(0u64);

    let mark_all_read = move |_| {
        spawn_local_scoped(async move {
            match mark_all_notifications_read().await {
                Ok(count) => {
                    leptos::logging::log!(
                        "[notifications] mark_all_notifications_read: marked {count}"
                    );
                    // Bump so stats refetch and the list remounts (stale rows
                    // would still show "Mark read" if only stats updated).
                    stats_trigger.update(|n| *n += 1);
                    // Clear optimistic overrides — remounted list is source of truth.
                    read_overrides.set(HashMap::new());
                }
                Err(e) => {
                    leptos::logging::warn!("Failed to mark all notifications read: {e}");
                }
            }
        });
    };

    view! {
        <div id="notifications-inbox-page">
            <ContentContainer data_testid="notifications-inbox-page" min_width=INBOX_MIN_WIDTH>
                <Flex vertical=true gap=SpacingSize::Size240.flex_gap()>
                    <Flex vertical=true gap=SpacingSize::Size80.flex_gap()>
                        <Title3>"Notifications"</Title3>
                    </Flex>

                    <NotificationStatsGrid trigger=stats_trigger />

                    <NotificationToolbar
                        search_query=search_query
                        filter=filter
                        on_mark_all_read=mark_all_read
                    />

                    <NotificationList
                        filter=filter
                        search_query=search_query.read_only()
                        read_overrides=read_overrides
                        reload=stats_trigger
                        navigate=navigate
                    />
                </Flex>
            </ContentContainer>
        </div>
    }
}
