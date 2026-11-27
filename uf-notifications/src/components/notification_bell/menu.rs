use crate::pages::inbox::{has_failed_page, LoadedRow};
use crate::server::NotificationRow;
use leptos::prelude::*;
use orbital_core_components::{List, ScrollArea};
use orbital_paging::UsePagedInfiniteScroll;
use uf_product::components::{Caption1, EmptyState, Subtitle2};
use uf_product::primitives::{
    Button, ButtonAppearance, ButtonSize, Flex, FlexAlign, FlexGap, FlexJustify, MenuItem,
    MessageBar, MessageBarActions, MessageBarBody, MessageBarIntent, Skeleton, SkeletonItem, Tab,
    TabList,
};

use super::item::NotificationBellItem;
use super::root::{TAB_ALL, TAB_UNREAD};

#[component]
pub fn NotificationBellDropdown(
    /// Selected tab: unread or all.
    tab: RwSignal<String>,
    /// Paged rows for the selected tab.
    paged: UsePagedInfiniteScroll<LoadedRow>,
    /// Bumped to refetch the unread count.
    trigger: RwSignal<u64>,
    /// Unread count, for the Unread tab label.
    count_res: Resource<Result<usize, ServerFnError>>,
    /// Whether to render times in the reader's local zone.
    local: Signal<bool>,
    /// Marks everything read.
    on_mark_all_read: Callback<()>,
    /// Opens notification preferences.
    on_preferences: Callback<()>,
    /// Reloads the list after a failed load.
    on_retry: Callback<()>,
) -> impl IntoView {
    let items = paged.items;
    let ever_loaded = paged.ever_loaded;
    let is_loading_more = paged.loading;
    let has_more = paged.has_more;
    let scroll_el = paged.scroll_el;

    let failed = Memo::new(move |_| items.with(|i| has_failed_page(i)));
    let rows = Memo::new(move |_| {
        items.with(|i| {
            i.iter()
                .flatten()
                .cloned()
                .collect::<Vec<NotificationRow>>()
        })
    });

    let unread_label = move || {
        count_res
            .get()
            .and_then(Result::ok)
            .map_or_else(|| "Unread".to_string(), |n| format!("Unread ({n})"))
    };

    let status = move || {
        if failed.get() {
            view! {
                <MessageBar intent=MessageBarIntent::Error attr:data-testid="notification-bell-error">
                    <MessageBarBody>"Couldn't load your notifications."</MessageBarBody>
                    <MessageBarActions>
                        <Button
                            size=ButtonSize::Small
                            on_click=Callback::new(move |_| on_retry.run(()))
                        >
                            "Try again"
                        </Button>
                    </MessageBarActions>
                </MessageBar>
            }
            .into_any()
        } else if !ever_loaded.get() {
            view! {
                <Flex vertical=true gap=FlexGap::Medium full_width=true attr:data-testid="notification-bell-skeleton">
                    {(0..3)
                        .map(|_| view! { <Skeleton><SkeletonItem /></Skeleton> })
                        .collect_view()}
                </Flex>
            }
            .into_any()
        } else if !rows.with(Vec::is_empty) {
            ().into_any()
        } else if tab.get() == TAB_ALL {
            view! { <EmptyState message="No notifications yet" /> }.into_any()
        } else {
            view! { <EmptyState message="You're all caught up" /> }.into_any()
        }
    };

    view! {
        <Flex vertical=true gap=FlexGap::Small full_width=true>
            <Flex justify=FlexJustify::SpaceBetween align=FlexAlign::Center full_width=true>
                <Subtitle2>"Notifications"</Subtitle2>
                <Flex gap=FlexGap::Small align=FlexAlign::Center>
                    <Button
                        size=ButtonSize::Small
                        appearance=ButtonAppearance::Subtle
                        on_click=Callback::new(move |_| on_mark_all_read.run(()))
                    >
                        "Mark all read"
                    </Button>
                    <Button
                        size=ButtonSize::Small
                        appearance=ButtonAppearance::Subtle
                        icon=icondata::AiSettingOutlined
                        attr:aria-label="Notification preferences"
                        on_click=Callback::new(move |_| on_preferences.run(()))
                    />
                </Flex>
            </Flex>
            <TabList selected_value=tab attr:data-testid="notification-bell-tabs">
                <Tab value=TAB_UNREAD>{unread_label}</Tab>
                <Tab value=TAB_ALL>"All"</Tab>
            </TabList>
            // ScrollArea documents `style` for bounded max-height (Orbital API, not a fork).
            <ScrollArea
                node_ref=scroll_el
                style="display: block; max-height: 340px; width: 100%; box-sizing: border-box;"
            >
                {status}
                <List>
                    <For
                        each=move || rows.get()
                        key=|row| (row.notification.notification_id, row.notification.is_read)
                        let:row
                    >
                        <NotificationBellItem
                            row=row
                            items=items
                            tab=tab
                            trigger=trigger
                            local=local
                        />
                    </For>
                </List>
                <Show when=move || is_loading_more.get() && has_more.get() && ever_loaded.get()>
                    <Caption1>"Loading more…"</Caption1>
                </Show>
            </ScrollArea>
        </Flex>
        <MenuItem value="view_all">"Open inbox"</MenuItem>
    }
}
