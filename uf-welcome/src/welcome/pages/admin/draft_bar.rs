//! "Draft" bar shown while the draft differs from the published list.

use leptos::prelude::*;
use uf_product::components::{MessageBarActions, MessageBarBody, MessageBarTitle};
use uf_product::primitives::{Button, ButtonAppearance, MessageBar, MessageBarIntent};

use crate::welcome::describe::describe_change;

use super::AdminState;

/// Says what changed and that members still see the old list, with Discard and Publish.
#[component]
pub(super) fn DraftBar(state: AdminState, publish_open: RwSignal<bool>) -> impl IntoView {
    let summary = Memo::new(move |_| {
        if !state.is_dirty() {
            return None;
        }
        let before = state.published_ids();
        let after = state.draft.get();
        Some(describe_change(&before, &after, |id| state.app_name(id)))
    });

    view! {
        {move || summary.get().map(|summary| view! {
            <div data-testid="featured-admin-draft-bar">
                <MessageBar intent=MessageBarIntent::Info>
                    <MessageBarBody>
                        <MessageBarTitle>"Draft"</MessageBarTitle>
                        {format!("You have unpublished changes: {summary}. Members still see the old list.")}
                    </MessageBarBody>
                    <MessageBarActions>
                        <Button
                            appearance=ButtonAppearance::Secondary
                            attr:data-testid="featured-admin-discard"
                            on_click=Callback::new(move |_| {
                                state.draft.set(state.published_ids());
                                state.conflict.set(None);
                            })
                        >
                            "Discard"
                        </Button>
                        <Button
                            appearance=ButtonAppearance::Primary
                            attr:data-testid="featured-admin-publish"
                            on_click=Callback::new(move |_| publish_open.set(true))
                        >
                            "Publish"
                        </Button>
                    </MessageBarActions>
                </MessageBar>
            </div>
        })}
    }
}
