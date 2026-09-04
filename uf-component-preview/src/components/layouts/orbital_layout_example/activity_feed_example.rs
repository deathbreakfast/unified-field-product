use leptos::prelude::*;
use uf_product::components::{
    Body1, Caption1, Card, MaterialElevation, MaterialVariant, SpacingSize, Subtitle2,
};
use uf_product::primitives::{
    Avatar, Button, ButtonAppearance, ButtonSize, Flex, FlexAlign, FlexJustify, Tooltip,
};

/// Layout example: an activity feed panel (preview-only placeholder).
#[component]
pub fn ActivityFeedExample() -> impl IntoView {
    view! {
        <Flex vertical=true gap=SpacingSize::Size160.flex_gap()>
            <Flex align=FlexAlign::Center justify=FlexJustify::SpaceBetween>
                <Subtitle2>"Activity Feed"</Subtitle2>
                <Flex gap=SpacingSize::Size80.flex_gap()>
                    <Tooltip content="Notifications">
                        <Button
                            appearance=ButtonAppearance::Subtle
                            size=ButtonSize::Small
                            icon=icondata::AiBellOutlined
                        />
                    </Tooltip>
                    <Tooltip content="Messages">
                        <Button
                            appearance=ButtonAppearance::Subtle
                            size=ButtonSize::Small
                            icon=icondata::BiChatRegular
                        />
                    </Tooltip>
                </Flex>
            </Flex>

            <Flex vertical=true gap=SpacingSize::Size120.flex_gap()>
                {activity_item("John Smith", "completed task 'UI Design Review'", "2 minutes ago")}
                {activity_item("Sarah Wilson", "uploaded new files to 'Mobile App' project", "15 minutes ago")}
                {activity_item("Mike Johnson", "commented on 'Website Redesign'", "1 hour ago")}
                {activity_item("Lisa Chen", "created new task 'Database Migration'", "3 hours ago")}
            </Flex>

            <Flex vertical=true gap=SpacingSize::Size80.flex_gap()>
                <Subtitle2>"Quick Actions"</Subtitle2>
                <Button appearance=ButtonAppearance::Secondary icon=icondata::AiPlusOutlined>
                    "New Project"
                </Button>
                <Button appearance=ButtonAppearance::Secondary icon=icondata::AiUserAddOutlined>
                    "Invite Member"
                </Button>
                <Button appearance=ButtonAppearance::Secondary icon=icondata::AiCalendarOutlined>
                    "Schedule Review"
                </Button>
            </Flex>
        </Flex>
    }
}

fn activity_item(name: &'static str, message: &'static str, time: &'static str) -> impl IntoView {
    view! {
        <Card variant=MaterialVariant::Outlined elevation=MaterialElevation::Flat>
            <Flex gap=SpacingSize::Size120.flex_gap() align=FlexAlign::FlexStart padding=SpacingSize::Size120.inset()>
                <Avatar />
                <Flex vertical=true gap=SpacingSize::Size40.flex_gap()>
                    <Body1>
                        <strong>{name}</strong>" "{message}
                    </Body1>
                    <Caption1>{time}</Caption1>
                </Flex>
            </Flex>
        </Card>
    }
}
