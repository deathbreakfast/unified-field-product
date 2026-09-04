use leptos::prelude::*;
use uf_product::components::{
    Body1, Caption1, Card, MaterialElevation, MaterialVariant, SpacingSize, Subtitle2,
};
use uf_product::primitives::{
    Button, ButtonAppearance, ButtonSize, Flex, FlexAlign, FlexJustify, Tooltip,
};

/// Layout example: a "Recently Shared" files overview card (preview-only placeholder).
#[component]
pub fn FilesOverviewExample() -> impl IntoView {
    view! {
        <Flex vertical=true gap=SpacingSize::Size160.flex_gap()>
            <Flex align=FlexAlign::Center justify=FlexJustify::SpaceBetween>
                <Subtitle2>"Recently Shared"</Subtitle2>
                <Tooltip content="Upload file">
                    <Button
                        appearance=ButtonAppearance::Subtle
                        size=ButtonSize::Small
                        icon=icondata::AiPlusOutlined
                    />
                </Tooltip>
            </Flex>

            <Flex vertical=true gap=SpacingSize::Size120.flex_gap()>
                {file_item("Roadmap-Q1.pdf", "Uploaded 10 minutes ago")}
                {file_item("App-assets.zip", "Shared by design team")}
                {file_item("Legal-review.docx", "Updated yesterday")}
            </Flex>
        </Flex>
    }
}

fn file_item(name: &'static str, meta: &'static str) -> impl IntoView {
    view! {
        <Card variant=MaterialVariant::Outlined elevation=MaterialElevation::Flat>
            <Flex vertical=true gap=SpacingSize::Size40.flex_gap() padding=SpacingSize::Size120.inset()>
                <Body1><strong>{name}</strong></Body1>
                <Caption1>{meta}</Caption1>
            </Flex>
        </Card>
    }
}
