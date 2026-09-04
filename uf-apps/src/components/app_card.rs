use leptos::prelude::*;
use leptos_router::components::A;
use turf::inline_style_sheet_values;
use uf_product::components::{
    Body1, Caption2, Card, CardContent, CardFooter, CardHeader, CardHeaderDescription, Subtitle1,
};
use uf_product::primitives::{Button, ButtonAppearance};

use crate::server::AppDirectoryItem;

fn truncate(text: &str, max: usize) -> String {
    if text.chars().count() > max {
        text.chars().take(max.saturating_sub(1)).collect::<String>() + "…"
    } else {
        text.to_string()
    }
}

/// A single app card in the apps directory grid.
#[component]
pub fn AppCard(
    /// App entry to display.
    app: AppDirectoryItem,
    /// When true, sets `id="apps-first-application-card"` for Help spotlight.
    #[prop(optional)]
    first: bool,
) -> impl IntoView {
    // Equal-height grid cells: card fills the cell; CardContent grows so Open stays bottom-right.
    let (style_sheet, class_names) = inline_style_sheet_values! {
        .Card {
            height: 100%;
        }

        .Content {
            flex: 1 1 auto;
        }

        .Link {
            text-decoration: none;
        }
    };

    let details_href = crate::paths::app(&app.slug);
    let prefetch_ctx = use_context::<crate::PrefetchAppFamily>();
    let route_for_enter = app.route_path.clone();
    let route_for_focus = app.route_path.clone();
    let prefetch_enter = prefetch_ctx.clone();
    let prefetch_focus = prefetch_ctx;

    let card_testid = format!("app-card-{}", app.slug);
    let card_id = first.then_some("apps-first-application-card");

    view! {
        <style>{style_sheet}</style>
        <div id=card_id data-testid=card_testid>
            <Card class=class_names.card>
                <CardHeader>
                    <Subtitle1>{app.name}</Subtitle1>
                    <CardHeaderDescription slot>
                        <Caption2>"Application"</Caption2>
                    </CardHeaderDescription>
                </CardHeader>
                <CardContent class=class_names.content>
                    <Body1 block=true wrap=true>
                        {truncate(&app.description, 140)}
                    </Body1>
                </CardContent>
                <CardFooter>
                    <A
                        href=details_href
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
                        <Button appearance=ButtonAppearance::Primary icon=icondata::AiArrowRightOutlined>
                            "Open"
                        </Button>
                    </A>
                </CardFooter>
            </Card>
        </div>
    }
}
