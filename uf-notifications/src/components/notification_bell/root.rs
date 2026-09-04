//! `NotificationBell` — the top-level composition component.

use crate::server::{get_unread_count, get_unread_notifications_page, subscribe_get_unread_count};
use crate::surface_layout::{BELL_DROPDOWN_MAX_WIDTH, BELL_DROPDOWN_MIN_WIDTH};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;
use orbital_paging::{page_fetch_from_tuple, use_paged_infinite_scroll};
use orbital_primitives::{Menu, MenuPosition, MenuTrigger};
use uf_product::use_auth_context;

use super::menu::NotificationBellDropdown;
use super::trigger::NotificationBellTrigger;

const PAGE_SIZE: u32 = 10;

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
    let refresh = RwSignal::new(0u32);
    let paged = use_paged_infinite_scroll(
        scroll_el,
        PAGE_SIZE,
        refresh.into(),
        page_fetch_from_tuple(get_unread_notifications_page),
    );

    let items = paged.items;
    let has_more = paged.has_more;
    let ever_loaded = paged.ever_loaded;
    let is_loading_more = paged.loading;

    let trigger = subscribe_get_unread_count(move || {
        refresh.update(|v| *v += 1);
    });

    let count_res = Resource::new(move || trigger.get(), move |_| get_unread_count());

    let navigate = use_navigate();
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
                items=items
                ever_loaded=ever_loaded
                is_loading_more=is_loading_more
                has_more=has_more
                scroll_el=scroll_el
                trigger=trigger
            />
        </Menu>
    }
}
