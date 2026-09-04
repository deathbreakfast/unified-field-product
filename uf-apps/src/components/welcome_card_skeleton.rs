use leptos::prelude::*;
use turf::inline_style_sheet_values;
use uf_product::components::{
    Caption1, Card, CardContent, CardFooter, CardHeader, CardHeaderDescription, CardSectionBorder,
    Skeleton, SkeletonItem, SpacingSize, Subtitle2, Title3,
};
use uf_product::primitives::{Button, ButtonAppearance, Flex, FlexAlign};

/// Loading skeleton for the app detail overview card.
#[component]
pub fn WelcomeCardSkeleton(
    /// Title text.
    title: String,
) -> impl IntoView {
    let (style_sheet, class_names) = inline_style_sheet_values! {
        .Card {
            width: 100%;
            max-width: 100%;
            margin: 0;
            box-sizing: border-box;
            min-height: 280px;
            display: flex;
            flex-direction: column;
        }

        .LineWide { width: 100%; height: 14px; border-radius: 6px; }
        .LineMed { width: 75%; height: 14px; border-radius: 6px; }

        .Content {
            flex: 1 1 auto;
        }

        .Footer {
            margin-top: auto;
        }

        .Tertiary {
            color: var(--orb-color-text-tertiary);
        }

        .Spacer {
            flex: 1 1 auto;
        }

        .LinkWrap {
            flex-wrap: wrap;
        }

        .FitContent {
            width: fit-content;
        }
    };

    view! {
        <style>{style_sheet}</style>
        <Card class=class_names.card>
            <CardHeader>
                <Title3>{title}</Title3>
                <CardHeaderDescription slot>
                    <Subtitle2>"Overview"</Subtitle2>
                </CardHeaderDescription>
            </CardHeader>
            <CardContent class=class_names.content>
                <Flex vertical=true gap=SpacingSize::Size100.flex_gap()>
                    <Skeleton><SkeletonItem class=class_names.line_wide /></Skeleton>
                    <Skeleton><SkeletonItem class=class_names.line_wide /></Skeleton>
                    <Skeleton><SkeletonItem class=class_names.line_med /></Skeleton>
                </Flex>
            </CardContent>

            <CardSectionBorder />
            <CardFooter class=class_names.footer>
                <Flex vertical=true gap=SpacingSize::Size80.flex_gap()>
                    <Caption1 class=class_names.tertiary>"Repositories & links"</Caption1>
                    <Flex align=FlexAlign::Center gap=SpacingSize::Size80.flex_gap() class=class_names.link_wrap>
                        <div class=class_names.spacer></div>
                        <div class=class_names.fit_content>
                            <Button
                                appearance=ButtonAppearance::Primary
                                icon=icondata::AiArrowRightOutlined
                                disabled=true
                                block=false
                            >
                                "Product link"
                            </Button>
                        </div>
                    </Flex>
                </Flex>
            </CardFooter>
        </Card>
    }
}
