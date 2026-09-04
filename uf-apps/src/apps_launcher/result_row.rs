//! One selectable row in the apps launcher results list.

use leptos::prelude::*;
use uf_product::components::{Body1, Caption1};
use uf_product::primitives::{Flex, FlexGap, ListItem};

use crate::server::AppDirectoryItem;

/// A single app hit in the launcher list. Click navigates via the parent callback.
#[component]
pub fn AppsLauncherResult(
    /// Registry directory item to display.
    app: AppDirectoryItem,
    /// Invoked with the app's `route_path` when the row is activated.
    on_select: Callback<String>,
) -> impl IntoView {
    let testid = format!("apps-launcher-result-{}", app.slug);
    let route_path = app.route_path.clone();
    let name = app.name.clone();
    let description = app.description.clone();

    view! {
        <div data-testid=testid>
            <ListItem on_click=Callback::new(move |_| {
                on_select.run(route_path.clone());
            })>
                <Flex vertical=true gap=FlexGap::Small full_width=true>
                    <Body1>{name}</Body1>
                    <Caption1>{description}</Caption1>
                </Flex>
            </ListItem>
        </div>
    }
}
