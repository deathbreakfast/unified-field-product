//! "Publish featured apps?" confirm.

use leptos::prelude::*;
use uf_product::components::{Body1, Caption1};
use uf_product::primitives::{
    Button, ButtonAppearance, Dialog, DialogActions, DialogBody, DialogContent, DialogSurface,
    DialogTitle, Flex, FlexGap,
};

use crate::welcome::describe::join_names;
use crate::welcome::server::{publish_featured, PublishFeaturedResult};

use super::AdminState;

/// Names the apps in order and publishes the draft on confirm.
#[component]
pub(super) fn PublishDialog(state: AdminState, open: RwSignal<bool>) -> impl IntoView {
    let busy = RwSignal::new(false);
    let message = Memo::new(move |_| {
        let names: Vec<String> = state
            .draft
            .get()
            .iter()
            .map(|id| state.app_name(id))
            .collect();
        if names.is_empty() {
            "Welcome hides the Featured card for every member the next time it loads.".to_string()
        } else {
            format!(
                "Every member sees {}, in that order, the next time Welcome loads.",
                join_names(&names)
            )
        }
    });

    let publish = move |_| {
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        let draft = state.draft.get_untracked();
        let based_on = state.version.get_untracked();
        leptos::task::spawn_local_scoped(async move {
            let result = publish_featured(draft.clone(), based_on).await;
            busy.set(false);
            open.set(false);
            match result {
                Ok(PublishFeaturedResult::Published { version, change_id }) => {
                    state.action_error.set(None);
                    state.conflict.set(None);
                    // Members now see the draft; the reload brings "Last published by".
                    let apps = draft.iter().filter_map(|id| state.app(id)).collect();
                    state.published.set(apps);
                    state.version.set(Some(version));
                    state.reload_fresh();
                    state.toast("Featured apps published.", change_id);
                }
                Ok(PublishFeaturedResult::Unchanged) => state.reload_fresh(),
                Ok(PublishFeaturedResult::Conflict { by_name }) => {
                    state.conflict.set(Some(by_name));
                }
                Err(err) => state.report(&err, "publish the featured apps"),
            }
        });
    };

    view! {
        <Dialog open=open>
            <DialogSurface attr:data-testid="featured-admin-publish-dialog">
                <DialogBody>
                    <DialogTitle>"Publish featured apps?"</DialogTitle>
                    <DialogContent>
                        <Flex vertical=true gap=FlexGap::Small>
                            <Body1>{move || message.get()}</Body1>
                            <Caption1>"You can undo this from History."</Caption1>
                        </Flex>
                    </DialogContent>
                    <DialogActions>
                        <Button
                            appearance=ButtonAppearance::Secondary
                            attr:data-testid="featured-admin-publish-cancel"
                            on_click=Callback::new(move |_| open.set(false))
                        >
                            "Cancel"
                        </Button>
                        <Button
                            appearance=ButtonAppearance::Primary
                            loading=busy
                            attr:data-testid="featured-admin-publish-confirm"
                            on_click=Callback::new(publish)
                        >
                            "Publish"
                        </Button>
                    </DialogActions>
                </DialogBody>
            </DialogSurface>
        </Dialog>
    }
}
