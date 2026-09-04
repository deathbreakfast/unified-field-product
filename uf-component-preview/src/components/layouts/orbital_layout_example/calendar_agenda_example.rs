use leptos::prelude::*;
use uf_product::components::{
    Body1, Caption1, Card, MaterialElevation, MaterialVariant, SpacingSize, Subtitle2,
};
use uf_product::primitives::{
    Button, ButtonAppearance, ButtonSize, Flex, FlexAlign, FlexJustify, Tooltip,
};

/// Layout example: an "Upcoming Events" agenda card (preview-only placeholder).
#[component]
pub fn CalendarAgendaExample() -> impl IntoView {
    view! {
        <Flex vertical=true gap=SpacingSize::Size160.flex_gap()>
            <Flex align=FlexAlign::Center justify=FlexJustify::SpaceBetween>
                <Subtitle2>"Upcoming Events"</Subtitle2>
                <Tooltip content="Add event">
                    <Button
                        appearance=ButtonAppearance::Subtle
                        size=ButtonSize::Small
                        icon=icondata::AiPlusOutlined
                    />
                </Tooltip>
            </Flex>

            <Flex vertical=true gap=SpacingSize::Size120.flex_gap()>
                {agenda_item("Product Design Review", "Today · 3:00 PM")}
                {agenda_item("Sprint Planning", "Tomorrow · 9:30 AM")}
                {agenda_item("Customer Advisory Board", "Fri · 1:00 PM")}
            </Flex>
        </Flex>
    }
}

fn agenda_item(title: &'static str, time: &'static str) -> impl IntoView {
    view! {
        <Card variant=MaterialVariant::Outlined elevation=MaterialElevation::Flat>
            <Flex vertical=true gap=SpacingSize::Size40.flex_gap() padding=SpacingSize::Size120.inset()>
                <Body1><strong>{title}</strong></Body1>
                <Caption1>{time}</Caption1>
            </Flex>
        </Card>
    }
}
