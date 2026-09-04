use crate::safe_url::safe_notification_nav_url;
use crate::server::{mark_notification_read, NotificationDto};
use leptos::prelude::*;
use leptos::task::spawn_local_scoped;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;
use uf_product::components::{
    Body1, Caption1, Card, CardButtonArea, CardContent, CardHeader, Subtitle1,
};
use uf_product::primitives::{
    Badge, BadgeColor, BadgeSize, Flex, FlexAlign, FlexGap, MaterialElevation, MaterialVariant,
};

#[component]
pub fn NotificationBellItem(
    /// Notification to render.
    notification: NotificationDto,
    /// Two-way signal holding the list of items to render.
    items: RwSignal<Vec<NotificationDto>>,
    /// Two-way signal holding the trigger element/state.
    trigger: RwSignal<u64>,
) -> impl IntoView {
    let notification_id = notification.notification_id;
    let url = notification.url.clone();
    let title = notification.title.clone();
    let message = notification.message.clone();
    let created_at = notification.created_at.clone();

    let open = Callback::new(move |_: leptos::ev::MouseEvent| {
        items.update(|list| list.retain(|n| n.notification_id != notification_id));

        spawn_local_scoped(async move {
            if let Err(e) = mark_notification_read(notification_id).await {
                leptos::logging::warn!("Failed to mark notification read: {e}");
            }
            trigger.update(|n| *n += 1);
        });

        let navigate = use_navigate();
        navigate(
            safe_notification_nav_url(url.as_deref()),
            NavigateOptions::default(),
        );
    });

    let (style_sheet, class_names) = turf::inline_style_sheet_values! {
        .Item {
            display: block;
            width: 100%;
            box-sizing: border-box;
        }

        .ItemCard {
            width: 100%;
            box-sizing: border-box;
        }

        .TitleRow {
            min-width: 0;
            flex: 1 1 0%;
        }

        .TitleText {
            overflow: hidden;
            text-overflow: ellipsis;
            white-space: nowrap;
            min-width: 0;
        }

        .MessageText {
            display: -webkit-box;
            -webkit-line-clamp: 2;
            -webkit-box-orient: vertical;
            overflow: hidden;
            min-width: 0;
        }
    };

    let item_cls = class_names.item.to_string();
    let card_cls = class_names.item_card.to_string();

    view! {
        <style>{style_sheet}</style>
        <div class=item_cls>
            <Card
                class=card_cls
                variant=MaterialVariant::Outlined
                elevation=MaterialElevation::Flat
                gap=FlexGap::Size(0)
            >
                <CardButtonArea on_click=open>
                    <CardHeader>
                        <Flex align=FlexAlign::Center gap=FlexGap::Small class=class_names.title_row.to_string()>
                            <Badge
                                size=Signal::from(BadgeSize::ExtraSmall)
                                color=Signal::from(BadgeColor::Danger)
                            />
                            <Subtitle1 class=class_names.title_text.to_string()>{title}</Subtitle1>
                        </Flex>
                    </CardHeader>
                    <CardContent>
                        <Flex vertical=true gap=FlexGap::Size(4)>
                            <Body1 block=true class=class_names.message_text.to_string()>{message}</Body1>
                            <Caption1>{created_at}</Caption1>
                        </Flex>
                    </CardContent>
                </CardButtonArea>
            </Card>
        </div>
    }
}
