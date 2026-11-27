use std::collections::HashMap;

use leptos::html::Div;
use leptos::prelude::*;
use leptos_router::NavigateOptions;
use orbital_core_components::{List, ScrollArea};
use orbital_paging::use_paged_infinite_scroll;
use uuid::Uuid;

use uf_product::components::{
    Caption1, Card, CardContent, EmptyState, EmptyStateCallToAction, SpacingSize, Subtitle2,
};
use uf_product::primitives::{
    Button, ButtonAppearance, ButtonSize, Flex, FlexGap, MessageBar, MessageBarActions,
    MessageBarBody, MessageBarIntent, MessageBarLayout, Skeleton, SkeletonItem,
};

use super::notification_item::{row_day_group, DayGroup};
use super::{fetch_rows, has_failed_page, InboxTab, NotificationItem, PREFERENCES_PATH};
use crate::server::{NotificationRow, PageRequest};

/// Number of notifications per page.
const NOTIFICATIONS_PAGE_SIZE: u32 = 20;

/// Rows (newest first) split into consecutive day groups.
fn group_rows(rows: &[NotificationRow], local: bool) -> Vec<(DayGroup, Vec<NotificationRow>)> {
    let mut groups: Vec<(DayGroup, Vec<NotificationRow>)> = Vec::new();
    for row in rows {
        let group = row_day_group(row.created_at, local);
        match groups.last_mut() {
            Some((last, members)) if *last == group => members.push(row.clone()),
            _ => groups.push((group, vec![row.clone()])),
        }
    }
    groups
}

/// Infinite-scrolling inbox list for the current tab, app and search, grouped
/// under Today, Yesterday and Earlier.
///
/// A change to the tab, app or search reloads the first page in place: rows
/// from the previous filter stay until the new page arrives, and nothing
/// around the list remounts.
#[component]
pub fn NotificationList(
    /// Selected tab (`InboxTab` value).
    tab: ReadSignal<String>,
    /// Selected app id; empty for all apps.
    app: ReadSignal<String>,
    /// Reactive signal for the search query.
    search_query: ReadSignal<String>,
    /// Two-way signal holding the read overrides.
    read_overrides: RwSignal<HashMap<Uuid, bool>>,
    /// Bumped after bulk ops (Mark all read) and by Try again to reload.
    reload: RwSignal<u64>,
    /// Navigation callback used to change routes.
    navigate: impl Fn(&str, NavigateOptions) + Clone + Send + Sync + 'static,
    /// `true` below `Md`.
    compact: Signal<bool>,
    /// Whether to render times and day groups in the reader's local zone.
    local: Signal<bool>,
    /// Resets tab, app and search.
    on_clear_filters: Callback<()>,
) -> impl IntoView {
    let scroll_el = NodeRef::<Div>::new();
    let refresh = Memo::new(move |prev: Option<&u32>| {
        tab.track();
        app.track();
        search_query.track();
        reload.track();
        prev.map_or(0, |n| n.wrapping_add(1))
    });
    let fetch = move |request: PageRequest| {
        let current = InboxTab::from_value(&tab.get_untracked());
        let app_id = Some(app.get_untracked()).filter(|a| !a.is_empty());
        let query = Some(search_query.get_untracked()).filter(|q| !q.trim().is_empty());
        fetch_rows(
            request,
            query,
            current.read_filter(),
            current.inbox_filter(app_id),
        )
    };
    let paged = use_paged_infinite_scroll(
        scroll_el,
        NOTIFICATIONS_PAGE_SIZE,
        Signal::derive(move || refresh.get()),
        fetch,
    );
    let items = paged.items;
    let ever_loaded = paged.ever_loaded;
    let has_more = paged.has_more;
    let loading = paged.loading;

    let failed = Memo::new(move |_| items.with(|i| has_failed_page(i)));
    let rows = Memo::new(move |_| {
        items.with(|i| {
            i.iter()
                .flatten()
                .cloned()
                .collect::<Vec<NotificationRow>>()
        })
    });
    let groups = Memo::new(move |_| rows.with(|r| group_rows(r, local.get())));
    let current_tab = Memo::new(move |_| InboxTab::from_value(&tab.get()));
    // The app filter or search narrowed the tab to nothing.
    let search_active =
        Memo::new(move |_| !app.get().is_empty() || !search_query.get().trim().is_empty());

    let prefs_navigate = navigate.clone();
    let open_preferences = Callback::new(move |()| {
        prefs_navigate(PREFERENCES_PATH, NavigateOptions::default());
    });
    let retry = Callback::new(move |()| reload.update(|n| *n += 1));

    let status = move || {
        if failed.get() {
            view! {
                <MessageBar
                    intent=MessageBarIntent::Error
                    layout=MessageBarLayout::Multiline
                    attr:data-testid="notifications-load-error"
                >
                    <MessageBarBody>
                        "Couldn't load your notifications. Nothing was lost; this is a problem on our side."
                    </MessageBarBody>
                    <MessageBarActions>
                        <Button
                            size=ButtonSize::Small
                            on_click=Callback::new(move |_| retry.run(()))
                        >
                            "Try again"
                        </Button>
                    </MessageBarActions>
                </MessageBar>
            }
            .into_any()
        } else if !ever_loaded.get() {
            view! { <InboxSkeleton /> }.into_any()
        } else if !rows.with(Vec::is_empty) {
            ().into_any()
        } else if search_active.get() {
            view! {
                <EmptyState
                    message="No notifications match"
                    description="Try another app or search."
                >
                    <EmptyStateCallToAction slot:call_to_action>
                        <Button
                            appearance=ButtonAppearance::Secondary
                            on_click=Callback::new(move |_| on_clear_filters.run(()))
                        >
                            "Clear filters"
                        </Button>
                    </EmptyStateCallToAction>
                </EmptyState>
            }
            .into_any()
        } else if current_tab.get() == InboxTab::Unread {
            view! { <EmptyState message="You're all caught up" description="Nothing unread." /> }
                .into_any()
        } else if current_tab.get() == InboxTab::Waiting {
            view! {
                <EmptyState
                    message="Nothing is waiting on you"
                    description="Notifications that ask you to do something show up here."
                />
            }
            .into_any()
        } else {
            view! {
                <EmptyState
                    message="No notifications yet"
                    description="When an app sends you something, it shows up here."
                >
                    <EmptyStateCallToAction slot:call_to_action>
                        <Button
                            appearance=ButtonAppearance::Secondary
                            on_click=Callback::new(move |_| open_preferences.run(()))
                        >
                            "Choose what you get"
                        </Button>
                    </EmptyStateCallToAction>
                </EmptyState>
            }
            .into_any()
        }
    };

    view! {
        // ScrollArea documents `style` for bounded max-height (Orbital API, not a fork).
        <ScrollArea
            node_ref=scroll_el
            style="display: block; max-height: 70vh; width: 100%; box-sizing: border-box;"
        >
            <Flex vertical=true gap=SpacingSize::Size160.flex_gap() full_width=true>
                {status}
                <For
                    each=move || groups.with(|gs| gs.iter().map(|(g, _)| *g).collect::<Vec<_>>())
                    key=|group| *group
                    let:group
                >
                    {
                        let navigate = navigate.clone();
                        let group_rows = Signal::derive(move || {
                            groups.with(|gs| {
                                gs.iter()
                                    .find(|(g, _)| *g == group)
                                    .map(|(_, members)| members.clone())
                                    .unwrap_or_default()
                            })
                        });
                        view! {
                            <DayGroupSection
                                group=group
                                rows=group_rows
                                read_overrides=read_overrides
                                navigate=navigate
                                compact=compact
                                local=local
                            />
                        }
                    }
                </For>
                <Show when=move || loading.get() && has_more.get() && ever_loaded.get()>
                    <Caption1>"Loading more…"</Caption1>
                </Show>
                <Show when=move || {
                    ever_loaded.get() && !has_more.get() && !failed.get() && !rows.with(Vec::is_empty)
                }>
                    <Caption1 attr:data-testid="notifications-end">
                        "That's everything from the last 90 days."
                    </Caption1>
                </Show>
            </Flex>
        </ScrollArea>
    }
}

/// One day heading with its rows in a card.
#[component]
fn DayGroupSection(
    /// Which day.
    group: DayGroup,
    /// Rows in this group, newest first.
    rows: Signal<Vec<NotificationRow>>,
    /// Two-way signal holding the read overrides.
    read_overrides: RwSignal<HashMap<Uuid, bool>>,
    /// Navigation callback used to change routes.
    navigate: impl Fn(&str, NavigateOptions) + Clone + Send + Sync + 'static,
    /// `true` below `Md`.
    compact: Signal<bool>,
    /// Whether to render times in the reader's local zone.
    local: Signal<bool>,
) -> impl IntoView {
    view! {
        <Flex
            vertical=true
            gap=FlexGap::Small
            full_width=true
            attr:data-testid="notifications-day-group"
            attr:data-group=group.label()
        >
            <Subtitle2>{group.label()}</Subtitle2>
            <Card>
                <CardContent>
                    <List>
                        <For
                            each=move || rows.get()
                            key=|row| (row.notification.notification_id, row.notification.is_read)
                            let:row
                        >
                            {
                                let navigate = navigate.clone();
                                view! {
                                    <NotificationItem
                                        row=row
                                        read_overrides=read_overrides
                                        navigate=navigate
                                        compact=compact
                                        local=local
                                    />
                                }
                            }
                        </For>
                    </List>
                </CardContent>
            </Card>
        </Flex>
    }
}

/// Placeholder rows while the first page loads.
#[component]
fn InboxSkeleton() -> impl IntoView {
    view! {
        <Card attr:data-testid="notifications-skeleton">
            <CardContent>
                <Flex vertical=true gap=FlexGap::Medium full_width=true>
                    {(0..3)
                        .map(|_| view! { <Skeleton><SkeletonItem /></Skeleton> })
                        .collect_view()}
                </Flex>
            </CardContent>
        </Card>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    fn row(title: &str, created_at: Option<chrono::DateTime<Utc>>) -> NotificationRow {
        NotificationRow {
            notification: crate::server::NotificationDto {
                notification_id: Uuid::new_v4(),
                title: title.into(),
                message: String::new(),
                created_at: "just now".into(),
                url: None,
                is_read: false,
                kind_id: None,
                kind_label: None,
                app_id: None,
                asks_action: false,
            },
            sender_name: "Someone".into(),
            created_at,
            app_name: None,
        }
    }

    #[test]
    fn group_rows_keeps_order_happy() {
        let now = Utc::now();
        let rows = vec![
            row("a", Some(now)),
            row("b", Some(now)),
            row("c", Some(now - Duration::days(5))),
        ];
        let groups = group_rows(&rows, false);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].0, DayGroup::Today);
        assert_eq!(groups[0].1.len(), 2);
        assert_eq!(groups[1].0, DayGroup::Earlier);
    }

    #[test]
    fn group_rows_empty_and_unknown_time_sad() {
        assert!(group_rows(&[], false).is_empty());
        let groups = group_rows(&[row("x", None)], false);
        assert_eq!(groups[0].0, DayGroup::Earlier);
    }
}
