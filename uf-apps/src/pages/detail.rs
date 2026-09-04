use leptos::prelude::*;
use leptos_router::{components::A, hooks::use_params_map};
use uf_product::components::ContentContainer;
use uf_product::primitives::{
    Button, ButtonAppearance, Flex, FlexGap, MessageBar, MessageBarIntent,
};

use crate::components::{AppOverviewCard, WelcomeCardSkeleton};
use crate::server::get_app_overview;

fn title_from_slug(slug: &str) -> String {
    if slug.is_empty() {
        return "App".to_string();
    }
    let mut chars = slug.chars();
    chars.next().map_or_else(
        || "App".to_string(),
        |first| first.to_uppercase().collect::<String>() + chars.as_str(),
    )
}

/// App detail page: overview card for a registered product app.
///
/// Reads `app_name` from the router, loads [`crate::server::get_app_overview`],
/// and renders the overview card (`AppOverviewCard`). Unknown slugs show a
/// warning [`uf_product::primitives::MessageBar`]; server-fn failures show an error banner.
#[component]
pub fn AppDetailPage() -> impl IntoView {
    let params = use_params_map();
    let slug = Memo::new(move |_| params.get().get("app_name").unwrap_or_default());
    let display_title = Memo::new(move |_| title_from_slug(&slug.get()));

    let overview_res = Resource::new(
        move || slug.get(),
        |app_name| async move { get_app_overview(app_name).await },
    );

    view! {
        <ContentContainer data_testid="app-detail-page">
            <Flex vertical=true gap=FlexGap::Large full_width=true>
                <A href=crate::paths::ROOT attr:style="text-decoration: none; width: fit-content;">
                    <Button appearance=ButtonAppearance::Secondary>"\u{2190} Back to apps"</Button>
                </A>

                <Suspense fallback=move || view! { <WelcomeCardSkeleton title=display_title.get_untracked() /> }>
                    {move || match overview_res.get() {
                        Some(Ok(Some(overview))) => {
                            view! { <AppOverviewCard overview=overview /> }.into_any()
                        }
                        Some(Ok(None)) => view! {
                            <MessageBar intent=MessageBarIntent::Warning>
                                "App not found."
                            </MessageBar>
                        }.into_any(),
                        Some(Err(err)) => view! {
                            <MessageBar intent=MessageBarIntent::Error>
                                "Failed to load app: " {err.to_string()}
                            </MessageBar>
                        }.into_any(),
                        None => view! { <WelcomeCardSkeleton title=display_title.get_untracked() /> }.into_any(),
                    }}
                </Suspense>
            </Flex>
        </ContentContainer>
    }
}
