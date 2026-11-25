mod notification_item;
mod notification_list;
mod toolbar;

pub use notification_item::NotificationItem;
pub use notification_list::NotificationList;
pub use toolbar::NotificationToolbar;

pub(crate) use notification_item::{
    fetch_rows, has_failed_page, item_style_sheet, mark_all_read, show_failure_toast,
    use_local_time, AppIcon, LoadedRow, NotificationMeta,
};

use std::collections::HashMap;

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;
use uf_product::components::{ContentContainer, SpacingSize, Title2};
use uf_product::primitives::{
    use_breakpoint_down, Breakpoint, Button, ButtonAppearance, Flex, FlexAlign, FlexGap,
    FlexJustify, FlexWrap, Tab, TabList, ToasterInjection,
};
use uuid::Uuid;

use crate::server::{
    get_inbox_counts, InboxCounts, NotificationInboxFilter, NotificationReadFilter,
};
use crate::surface_layout::INBOX_MIN_WIDTH;
use notification_item::CountsRefresh;

/// Notification preferences page (owned by the preferences work; may 404 until it lands).
pub(crate) const PREFERENCES_PATH: &str = "/notifications/preferences";

/// The inbox tabs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InboxTab {
    /// Every notification.
    #[default]
    All,
    /// Not yet read.
    Unread,
    /// Kinds that ask the reader to act.
    Waiting,
}

impl InboxTab {
    /// `TabList` value.
    pub const fn value(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Unread => "unread",
            Self::Waiting => "waiting",
        }
    }

    /// Tab for a `TabList` value; unknown values are [`InboxTab::All`].
    pub fn from_value(value: &str) -> Self {
        match value {
            "unread" => Self::Unread,
            "waiting" => Self::Waiting,
            _ => Self::All,
        }
    }

    /// Read filter the tab applies.
    pub const fn read_filter(self) -> NotificationReadFilter {
        match self {
            Self::Unread => NotificationReadFilter::Unread,
            Self::All | Self::Waiting => NotificationReadFilter::All,
        }
    }

    /// Inbox filter for this tab and an optional app.
    pub fn inbox_filter(self, app_id: Option<String>) -> NotificationInboxFilter {
        NotificationInboxFilter {
            asks_action: self == Self::Waiting,
            app_id,
        }
    }
}

/// "All (5)", or just "All" while the counts load or when they failed.
fn tab_label(name: &str, count: Option<usize>) -> String {
    count.map_or_else(|| name.to_string(), |n| format!("{name} ({n})"))
}

/// Full notification inbox: tabs with counts, search and app filter, and a
/// paginated list grouped by day, with optimistic read-state overrides.
///
/// Switching tabs, the app filter or the search refetches only the list; the
/// header, tabs and toolbar stay mounted.
#[component]
pub fn NotificationsInboxPage() -> impl IntoView {
    let tab = RwSignal::new(InboxTab::All.value().to_string());
    let app = RwSignal::new(String::new());
    let search_query = RwSignal::new(String::new());
    let read_overrides = RwSignal::new(HashMap::<Uuid, bool>::new());
    let navigate = use_navigate();
    let local = use_local_time();
    let compact: Signal<bool> = use_breakpoint_down(Breakpoint::Md).into();

    // Bumped after Mark all read so the list reloads and the counts refetch.
    let reload = RwSignal::new(0u64);
    // Bumped after a single read change so only the counts refetch.
    let counts_refresh = RwSignal::new(0u64);
    provide_context(CountsRefresh(counts_refresh));

    // Client-only: the counts are labels inside the tabs, so SSR renders the
    // bare names and hydration matches.
    let counts = LocalResource::new(move || {
        let _ = reload.get();
        let _ = counts_refresh.get();
        get_inbox_counts()
    });
    let count_of =
        move |pick: fn(&InboxCounts) -> usize| counts.get().and_then(Result::ok).map(|c| pick(&c));

    let toaster = use_context::<ToasterInjection>();
    let on_mark_all_read = Callback::new(move |()| {
        mark_all_read(
            toaster,
            Callback::new(move |()| {
                // The reloaded list is the source of truth again.
                read_overrides.set(HashMap::new());
                reload.update(|n| *n += 1);
            }),
        );
    });

    let clear_filters = Callback::new(move |()| {
        tab.set(InboxTab::All.value().to_string());
        app.set(String::new());
        search_query.set(String::new());
    });

    let prefs_navigate = navigate.clone();

    view! {
        <div id="notifications-inbox-page">
            <ContentContainer data_testid="notifications-inbox-page" min_width=INBOX_MIN_WIDTH>
                <Flex vertical=true gap=SpacingSize::Size240.flex_gap()>
                    <Flex
                        justify=FlexJustify::SpaceBetween
                        align=FlexAlign::Center
                        wrap=FlexWrap::Wrap
                        gap=FlexGap::Medium
                        full_width=true
                    >
                        <Title2>"Notifications"</Title2>
                        <Flex gap=FlexGap::Small>
                            <Button
                                appearance=ButtonAppearance::Secondary
                                on_click=Callback::new(move |_| on_mark_all_read.run(()))
                            >
                                "Mark all read"
                            </Button>
                            <Button
                                appearance=ButtonAppearance::Secondary
                                icon=icondata::AiSettingOutlined
                                on_click=Callback::new(move |_| {
                                    prefs_navigate(PREFERENCES_PATH, NavigateOptions::default());
                                })
                            >
                                "Preferences"
                            </Button>
                        </Flex>
                    </Flex>

                    <TabList selected_value=tab attr:data-testid="notifications-inbox-tabs">
                        <Tab value=InboxTab::All.value()>
                            {move || tab_label("All", count_of(|c| c.all))}
                        </Tab>
                        <Tab value=InboxTab::Unread.value()>
                            {move || tab_label("Unread", count_of(|c| c.unread))}
                        </Tab>
                        <Tab value=InboxTab::Waiting.value()>
                            {move || tab_label("Waiting on you", count_of(|c| c.waiting))}
                        </Tab>
                    </TabList>

                    <NotificationToolbar search_query=search_query app=app />

                    <NotificationList
                        tab=tab.read_only()
                        app=app.read_only()
                        search_query=search_query.read_only()
                        read_overrides=read_overrides
                        reload=reload
                        navigate=navigate
                        compact=compact
                        local=local
                        on_clear_filters=clear_filters
                    />
                </Flex>
            </ContentContainer>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inbox_tab_round_trips_happy() {
        for tab in [InboxTab::All, InboxTab::Unread, InboxTab::Waiting] {
            assert_eq!(InboxTab::from_value(tab.value()), tab);
        }
    }

    #[test]
    fn inbox_tab_unknown_value_is_all_sad() {
        assert_eq!(InboxTab::from_value("read"), InboxTab::All);
        assert_eq!(InboxTab::from_value(""), InboxTab::All);
    }

    #[test]
    fn inbox_tab_filters_happy() {
        assert_eq!(
            InboxTab::Unread.read_filter(),
            NotificationReadFilter::Unread
        );
        assert_eq!(InboxTab::Waiting.read_filter(), NotificationReadFilter::All);
        let waiting = InboxTab::Waiting.inbox_filter(Some("magnon".into()));
        assert!(waiting.asks_action);
        assert_eq!(waiting.app_id.as_deref(), Some("magnon"));
        assert!(!InboxTab::All.inbox_filter(None).is_active());
    }

    #[test]
    fn tab_label_counts_happy() {
        assert_eq!(tab_label("All", Some(5)), "All (5)");
        assert_eq!(tab_label("Waiting on you", None), "Waiting on you");
    }
}
