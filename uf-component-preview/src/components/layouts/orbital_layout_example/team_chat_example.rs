use leptos::prelude::*;
use uf_product::components::{
    Body1, Caption1, Card, MaterialElevation, MaterialVariant, SpacingSize, Subtitle2,
};
use uf_product::primitives::{
    Avatar, Button, ButtonAppearance, ButtonSize, Flex, FlexAlign, FlexJustify, Tooltip,
};

/// Layout example: a team chat panel (preview-only placeholder).
#[component]
pub fn TeamChatExample() -> impl IntoView {
    view! {
        <Flex vertical=true gap=SpacingSize::Size160.flex_gap()>
            <Flex align=FlexAlign::Center justify=FlexJustify::SpaceBetween>
                <Subtitle2>"Team Chat"</Subtitle2>
                <Tooltip content="Start new thread">
                    <Button
                        appearance=ButtonAppearance::Subtle
                        size=ButtonSize::Small
                        icon=icondata::AiPlusOutlined
                    />
                </Tooltip>
            </Flex>

            <Flex vertical=true gap=SpacingSize::Size120.flex_gap()>
                {chat_message("Dana Rodgers", "Could someone review the analytics dashboard PR?", "Just now")}
                {chat_message("Marketing Team", "Shared campaign assets in Files.", "12 minutes ago")}
            </Flex>

            <Flex vertical=true gap=SpacingSize::Size80.flex_gap()>
                <Subtitle2>"Quick Replies"</Subtitle2>
                <Button appearance=ButtonAppearance::Secondary>"👍 On it"</Button>
                <Button appearance=ButtonAppearance::Secondary>"📎 Share update"</Button>
                <Button appearance=ButtonAppearance::Secondary>"⏱ Need more time"</Button>
            </Flex>
        </Flex>
    }
}

fn chat_message(name: &'static str, message: &'static str, time: &'static str) -> impl IntoView {
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
