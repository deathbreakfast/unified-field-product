use crate::pages::inbox::{
    item_style_sheet, show_failure_toast, AppIcon, LoadedRow, NotificationMeta,
};
use crate::safe_url::safe_notification_nav_url;
use crate::server::{mark_notification_read, NotificationRow};
use leptos::prelude::*;
use leptos::task::spawn_local_scoped;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;
use orbital_core_components::ListItem;
use uf_product::components::{Body1, Body1Strong};
use uf_product::primitives::{Badge, BadgeSize, Flex, FlexAlign, FlexGap, ToasterInjection};
use uuid::Uuid;

use super::root::TAB_UNREAD;

/// Mark one notification read from the bell. On failure the row comes back
/// (the list reloads) and a toast names it, with Try again.
fn bell_mark_read(
    id: Uuid,
    title: String,
    trigger: RwSignal<u64>,
    toaster: Option<ToasterInjection>,
) {
    spawn_local_scoped(async move {
        if let Err(e) = mark_notification_read(id).await {
            leptos::logging::warn!("Failed to mark notification read: {e}");
            let retry_title = title.clone();
            show_failure_toast(
                toaster,
                format!("Couldn't mark “{title}” as read"),
                "It's still unread.",
                Callback::new(move |()| {
                    bell_mark_read(id, retry_title.clone(), trigger, toaster);
                }),
            );
        }
        let _ = trigger.try_update(|n| *n += 1);
    });
}

#[component]
pub fn NotificationBellItem(
    /// Notification to render, with sender, time and app.
    row: NotificationRow,
    /// Loaded rows; opening an unread row removes it from the Unread tab.
    items: RwSignal<Vec<LoadedRow>>,
    /// Selected bell tab.
    tab: RwSignal<String>,
    /// Bumped to refetch the unread count (and the list).
    trigger: RwSignal<u64>,
    /// Whether to render times in the reader's local zone.
    local: Signal<bool>,
) -> impl IntoView {
    let notification = row.notification.clone();
    let notification_id = notification.notification_id;
    let is_read = notification.is_read;
    let url = notification.url.clone();
    let title = notification.title.clone();
    let message = notification.message.clone();
    let toaster = use_context::<ToasterInjection>();

    let open_title = title.clone();
    let open = Callback::new(move |()| {
        if tab.get_untracked() == TAB_UNREAD {
            items.update(|list| {
                list.retain(|r| {
                    r.as_ref()
                        .is_none_or(|r| r.notification.notification_id != notification_id)
                });
            });
        }
        if !is_read {
            bell_mark_read(notification_id, open_title.clone(), trigger, toaster);
        }

        let navigate = use_navigate();
        navigate(
            safe_notification_nav_url(url.as_deref()),
            NavigateOptions::default(),
        );
    });

    let (style_sheet, class_names) = item_style_sheet();
    let dot_class = if is_read {
        format!(
            "{} {}",
            class_names.unread_dot, class_names.unread_dot_hidden
        )
    } else {
        class_names.unread_dot.clone()
    };
    let title_view = if is_read {
        view! { <Body1>{title}</Body1> }.into_any()
    } else {
        view! { <Body1Strong>{title}</Body1Strong> }.into_any()
    };

    view! {
        <style>{style_sheet}</style>
        <ListItem on_click=open attr:data-testid="notification-bell-item">
            <Flex align=FlexAlign::Start gap=FlexGap::Medium full_width=true>
                <AppIcon app_name=row.app_name.clone() />
                <Flex vertical=true gap=FlexGap::Size(4) class=class_names.content>
                    <Flex align=FlexAlign::Center gap=FlexGap::Small>
                        <Badge size=Signal::from(BadgeSize::ExtraSmall) class=dot_class />
                        {title_view}
                    </Flex>
                    <Body1 block=true class=class_names.message_text>{message}</Body1>
                    <NotificationMeta row=row local=local />
                </Flex>
            </Flex>
        </ListItem>
    }
}
