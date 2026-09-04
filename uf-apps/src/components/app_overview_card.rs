//! Apps overview card for the detail page.

use leptos::prelude::*;
use leptos_router::components::A;
use turf::inline_style_sheet_values;
use uf_product::components::{
    Body1, Caption1, Card, CardContent, CardFooter, CardHeader, CardHeaderDescription,
    CardSectionBorder, SpacingSize, Subtitle2, Title3,
};
use uf_product::primitives::{Button, ButtonAppearance, Flex, FlexAlign, Tooltip};

use crate::server::AppOverview;

/// Docs.rs URL for a crates.io package name.
fn docs_rs_url(crate_name: &str) -> String {
    format!("https://docs.rs/{crate_name}")
}

/// The main overview card on the app detail page.
#[component]
pub fn AppOverviewCard(
    /// Overview data to display.
    overview: AppOverview,
) -> impl IntoView {
    let (style_sheet, class_names) = inline_style_sheet_values! {
        .Card {
            width: 100%;
            max-width: 100%;
            margin: 0;
            box-sizing: border-box;
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

        .Link {
            text-decoration: none;
        }

        .ExternalLink {
            text-decoration: none;
            color: inherit;
            display: inline-flex;
        }
    };

    let repository = overview.repository.clone();
    let crate_name = overview.crate_name.clone();
    let docs_href = crate_name.as_ref().map(|name| docs_rs_url(name));

    view! {
        <style>{style_sheet}</style>
        <Card class=class_names.card>
            <CardHeader>
                <Title3>{overview.name}</Title3>
                <CardHeaderDescription slot>
                    <Subtitle2>"Overview"</Subtitle2>
                </CardHeaderDescription>
            </CardHeader>
            <CardContent>
                <div id="app-overview-more-information">
                    <Body1 block=true wrap=true>{overview.description}</Body1>
                </div>
            </CardContent>

            <CardSectionBorder />
            <CardFooter>
                <Flex vertical=true gap=SpacingSize::Size80.flex_gap() full_width=true>
                    <Caption1 class=class_names.tertiary>"Repositories & links"</Caption1>
                    <Flex align=FlexAlign::Center gap=SpacingSize::Size80.flex_gap() class=class_names.link_wrap full_width=true>
                        {
                            match repository {
                                Some(url) => view! {
                                    <span id="app-overview-source-code">
                                        <Tooltip content="GitHub repository">
                                            <a
                                                href=url
                                                target="_blank"
                                                rel="noopener noreferrer"
                                                class=class_names.external_link
                                                data-testid="app-overview-github"
                                            >
                                                <Button
                                                    appearance=ButtonAppearance::Subtle
                                                    icon=icondata::AiGithubOutlined
                                                    attr:aria-label="GitHub repository"
                                                />
                                            </a>
                                        </Tooltip>
                                    </span>
                                }.into_any(),
                                None => ().into_any(),
                            }
                        }
                        {
                            match docs_href {
                                Some(url) => view! {
                                    <span id="app-overview-documentation">
                                        <Tooltip content="Documentation on docs.rs">
                                            <a
                                                href=url
                                                target="_blank"
                                                rel="noopener noreferrer"
                                                class=class_names.external_link
                                                data-testid="app-overview-docs"
                                            >
                                                <Button
                                                    appearance=ButtonAppearance::Subtle
                                                    icon=icondata::AiFileTextOutlined
                                                    attr:aria-label="Documentation"
                                                />
                                            </a>
                                        </Tooltip>
                                    </span>
                                }.into_any(),
                                None => ().into_any(),
                            }
                        }

                        <div class=class_names.spacer></div>

                        <div id="app-overview-product-link" class=class_names.fit_content>
                            {
                                let prefetch_ctx = use_context::<crate::PrefetchAppFamily>();
                                let route_for_enter = overview.route_path.clone();
                                let route_for_focus = overview.route_path.clone();
                                let href = overview.route_path.clone();
                                let prefetch_enter = prefetch_ctx.clone();
                                let prefetch_focus = prefetch_ctx;
                                view! {
                                    <A
                                        href=href
                                        attr:class=class_names.link
                                        on:mouseenter=move |_| {
                                            if let Some(crate::PrefetchAppFamily(prefetch)) = prefetch_enter.as_ref() {
                                                prefetch(&route_for_enter);
                                            }
                                        }
                                        on:focus=move |_| {
                                            if let Some(crate::PrefetchAppFamily(prefetch)) = prefetch_focus.as_ref() {
                                                prefetch(&route_for_focus);
                                            }
                                        }
                                    >
                                        <Button
                                            appearance=ButtonAppearance::Primary
                                            icon=icondata::AiArrowRightOutlined
                                            block=false
                                        >
                                            "Product link"
                                        </Button>
                                    </A>
                                }
                            }
                        </div>
                    </Flex>
                </Flex>
            </CardFooter>
        </Card>
    }
}

#[cfg(test)]
mod tests {
    use super::docs_rs_url;

    #[test]
    fn docs_rs_url_uses_crate_name_happy_path() {
        assert_eq!(docs_rs_url("uf-apps"), "https://docs.rs/uf-apps");
    }
}
