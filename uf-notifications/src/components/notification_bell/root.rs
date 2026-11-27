//! `NotificationBell` — the top-level composition component.

use crate::pages::inbox::{fetch_rows, mark_all_read, use_local_time, PREFERENCES_PATH};
use crate::server::{
    get_unread_count, subscribe_get_unread_count, NotificationInboxFilter, NotificationReadFilter,
    PageRequest,
};
use crate::surface_layout::{BELL_DROPDOWN_MAX_WIDTH, BELL_DROPDOWN_MIN_WIDTH};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;
use orbital_paging::use_paged_infinite_scroll;
use orbital_primitives::{Menu, MenuPosition, MenuTrigger};
use uf_product::use_auth_context;

use super::menu::NotificationBellDropdown;
use super::trigger::NotificationBellTrigger;

const PAGE_SIZE: u32 = 10;

/// Bell tab showing only unread notifications.
pub(super) const TAB_UNREAD: &str = "unread";
/// Bell tab showing every notification.
pub(super) const TAB_ALL: &str = "all";

/// Notification bell for authenticated sessions only.
#[component]
pub fn NotificationBell() -> impl IntoView {
    let auth = use_auth_context();
    let is_authenticated = Memo::new(move |_| {
        auth.session()
            .with(uf_product::AuthSession::is_authenticated)
    });

    view! {
        <Show when=move || is_authenticated.get()>
            <NotificationBellAuthenticated />
        </Show>
    }
}

#[component]
fn NotificationBellAuthenticated() -> impl IntoView {
    let scroll_el = NodeRef::<leptos::html::Div>::new();
    let tab = RwSignal::new(TAB_UNREAD.to_string());
    let reload = RwSignal::new(0u32);
    let refresh = Memo::new(move |prev: Option<&u32>| {
        tab.track();
        reload.track();
        prev.map_or(0, |n| n.wrapping_add(1))
    });
    let paged = use_paged_infinite_scroll(
        scroll_el,
        PAGE_SIZE,
        Signal::derive(move || refresh.get()),
        move |request: PageRequest| {
            let read_filter = if tab.get_untracked() == TAB_ALL {
                NotificationReadFilter::All
            } else {
                NotificationReadFilter::Unread
            };
            fetch_rows(
                request,
                None,
                read_filter,
                NotificationInboxFilter::default(),
            )
        },
    );

    let trigger = subscribe_get_unread_count(move || {
        reload.update(|v| *v += 1);
    });

    let count_res = Resource::new(move || trigger.get(), move |_| get_unread_count());
    let local = use_local_time();

    let toaster = use_context::<uf_product::primitives::ToasterInjection>();
    let on_mark_all_read = Callback::new(move |()| {
        mark_all_read(
            toaster,
            Callback::new(move |()| {
                trigger.update(|n| *n += 1);
                reload.update(|v| *v += 1);
            }),
        );
    });
    let on_retry = Callback::new(move |()| reload.update(|v| *v += 1));

    let navigate = use_navigate();
    let prefs_navigate = navigate.clone();
    let on_preferences = Callback::new(move |()| {
        prefs_navigate(PREFERENCES_PATH, NavigateOptions::default());
    });
    let handle_select = move |key: &str| {
        if key == "view_all" {
            navigate(super::NOTIFICATIONS_PATH, NavigateOptions::default());
        }
    };

    view! {
        <Menu
            on_select=handle_select
            position=MenuPosition::BottomEnd
            min_width=BELL_DROPDOWN_MIN_WIDTH
            max_width=BELL_DROPDOWN_MAX_WIDTH
            data_testid="notification-bell-dropdown"
        >
            <MenuTrigger slot>
                <NotificationBellTrigger count_res=count_res />
            </MenuTrigger>
            <NotificationBellDropdown
                tab=tab
                paged=paged
                trigger=trigger
                count_res=count_res
                local=local
                on_mark_all_read=on_mark_all_read
                on_preferences=on_preferences
                on_retry=on_retry
            />
        </Menu>
    }
}
