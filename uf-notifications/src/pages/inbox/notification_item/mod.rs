mod helpers;

use std::collections::HashMap;

use leptos::ev;
use leptos::prelude::*;
use leptos::task::spawn_local_scoped;
use uuid::Uuid;

use leptos_router::NavigateOptions;
use uf_product::components::{
    Body1, Caption1, Card, CardButtonArea, CardContent, CardFooter, CardHeader, Subtitle1,
};
use uf_product::primitives::{
    Badge, BadgeSize, Button, ButtonAppearance, ButtonSize, Flex, FlexAlign, FlexGap,
    MaterialElevation, MaterialVariant,
};

use crate::safe_url::safe_notification_nav_url;
use crate::server::{mark_notification_read, mark_notification_unread, NotificationDto};

use helpers::{item_style_sheet, mark_read_label};

/// A single notification row in the inbox list.
///
/// Derives read/unread state reactively from `read_overrides` so the item
/// updates in-place when toggled (compatible with `<For>` keyed rendering).
#[component]
#[allow(clippy::needless_pass_by_value)] // Leptos `#[component]` props must be owned.
pub fn NotificationItem(
    /// Notification to render.
    notification: NotificationDto,
    /// Two-way signal holding the read overrides.
    read_overrides: RwSignal<HashMap<Uuid, bool>>,
    /// Navigation callback used to change routes.
    navigate: impl Fn(&str, NavigateOptions) + Clone + Send + Sync + 'static,
) -> impl IntoView {
    let notification_id = notification.notification_id;
    let original_is_read = notification.is_read;
    let url = notification.url.clone();
    let title = notification.title.clone();
    let message = notification.message.clone();
    let created_at = notification.created_at;

    let is_read = Memo::new(move |_| {
        read_overrides
            .get()
            .get(&notification_id)
            .copied()
            .unwrap_or(original_is_read)
    });

    let (style_sheet, class_names) = item_style_sheet();
    let item_read_cls = class_names.item_read.clone();
    let card_class = Signal::derive(move || {
        if is_read.get() {
            item_read_cls.clone()
        } else {
            String::new()
        }
    });

    let dot_base = class_names.unread_dot.clone();
    let dot_hidden_cls = class_names.unread_dot_hidden.clone();
    let dot_class = Signal::derive(move || {
        if is_read.get() {
            format!("{dot_base} {dot_hidden_cls}")
        } else {
            dot_base.clone()
        }
    });

    let open_notification = Callback::new(move |_: ev::MouseEvent| {
        read_overrides.update(|overrides| {
            overrides.insert(notification_id, true);
        });
        spawn_local_scoped(async move {
            if let Err(e) = mark_notification_read(notification_id).await {
                leptos::logging::warn!("Failed to mark notification read: {e}");
            }
        });
        if url.is_some() {
            navigate(
                safe_notification_nav_url(url.as_deref()),
                NavigateOptions::default(),
            );
        }
    });

    view! {
        <style>{style_sheet}</style>
        <Card
            class=card_class
            variant=MaterialVariant::Outlined
            elevation=MaterialElevation::Flat
            gap=FlexGap::Size(0)
        >
            <Flex
                class=class_names.row
                full_width=true
                align=FlexAlign::Stretch
                gap=FlexGap::Size(0)
            >
                <CardButtonArea class=class_names.hit_fill on_click=open_notification>
                    <CardHeader>
                        <Flex align=FlexAlign::Center gap=FlexGap::Small>
                            <Badge size=Signal::from(BadgeSize::ExtraSmall) class=dot_class />
                            <Subtitle1>{title}</Subtitle1>
                        </Flex>
                    </CardHeader>
                    <CardContent>
                        <Flex vertical=true gap=FlexGap::Size(4)>
                            <Body1 block=true wrap=true>{message}</Body1>
                            <Caption1>{created_at}</Caption1>
                        </Flex>
                    </CardContent>
                </CardButtonArea>
                <CardFooter class=class_names.side_footer>
                    <Button
                        size=ButtonSize::Small
                        appearance=ButtonAppearance::Subtle
                        on_click=Callback::new(move |ev: ev::MouseEvent| {
                            ev.stop_propagation();
                            let current = read_overrides
                                .get_untracked()
                                .get(&notification_id)
                                .copied()
                                .unwrap_or(original_is_read);
                            let new_is_read = !current;
                            read_overrides.update(|overrides| {
                                overrides.insert(notification_id, new_is_read);
                            });
                            spawn_local_scoped(async move {
                                let result = if new_is_read {
                                    mark_notification_read(notification_id).await
                                } else {
                                    mark_notification_unread(notification_id).await
                                };
                                if let Err(e) = result {
                                    leptos::logging::warn!(
                                        "Failed to toggle notification read state: {e}"
                                    );
                                }
                            });
                        })
                    >
                        {move || mark_read_label(is_read.get())}
                    </Button>
                </CardFooter>
            </Flex>
        </Card>
    }
}
