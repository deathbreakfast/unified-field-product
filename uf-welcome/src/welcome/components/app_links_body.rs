//! Shared list body for welcome app-link cards.

use leptos::prelude::*;
use leptos::tachys::view::any_view::IntoAny;
use leptos_router::components::A;
use turf::inline_style_sheet_values;
use uf_product::components::{Body1, Skeleton, SkeletonItem};
use uf_product::primitives::{MessageBar, MessageBarIntent};

use crate::welcome::server::AppLinkDto;

/// Loading skeleton for an app-link card body.
#[component]
pub fn AppLinksSkeleton() -> impl IntoView {
    let (style_sheet, class_names) = inline_style_sheet_values! {
        .SkeletonItem {
            width: 100%;
            height: 24px;
            border-radius: 4px;
        }

        .SkeletonContainer {
            display: flex;
            flex-direction: column;
            gap: 8px;
        }
    };

    view! {
        <style>{style_sheet}</style>
        <div class=class_names.skeleton_container>
            <Skeleton><SkeletonItem class=class_names.skeleton_item /></Skeleton>
            <Skeleton><SkeletonItem class=class_names.skeleton_item /></Skeleton>
        </div>
    }
}

/// Render a resource of app links with empty / error handling.
#[component]
pub fn AppLinksBody(
    /// Async resource yielding the link list.
    links: Resource<Result<Vec<AppLinkDto>, ServerFnError>>,
    /// Empty-state copy when the list is empty.
    empty_message: &'static str,
    /// Prefix for error message.
    error_label: &'static str,
) -> impl IntoView {
    let (style_sheet, class_names) = inline_style_sheet_values! {
        .AppLinkItem {
            display: flex;
            align-items: center;
            gap: 8px;
        }
    };

    view! {
        <style>{style_sheet}</style>
        <Suspense fallback=move || view! { <AppLinksSkeleton /> }>
            {move || match links.get() {
                Some(Ok(apps)) => {
                    if apps.is_empty() {
                        view! {
                            <div data-testid="app-links-empty">
                                <Body1>{empty_message}</Body1>
                            </div>
                        }
                        .into_any()
                    } else {
                        view! {
                            <div data-testid="app-links-list">
                                {apps
                                    .into_iter()
                                    .map(|app| {
                                        let test_id = format!("app-link-{}", app.app_id);
                                        view! {
                                            <div class=class_names.app_link_item data-testid=test_id>
                                                <A href=app.link.clone() attr:style="text-decoration: none; flex-grow: 1;">
                                                    <Body1>{app.label}</Body1>
                                                </A>
                                            </div>
                                        }
                                    })
                                    .collect_view()}
                            </div>
                        }
                        .into_any()
                    }
                }
                Some(Err(err)) => view! {
                    <MessageBar intent=MessageBarIntent::Error>
                        {format!("{error_label}: {err}")}
                    </MessageBar>
                }
                .into_any(),
                None => ().into_any(),
            }}
        </Suspense>
    }
}
