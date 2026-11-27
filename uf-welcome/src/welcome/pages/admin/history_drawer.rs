//! "Featured apps history" drawer with Restore.

use leptos::prelude::*;
use uf_product::components::{
    Caption1, EmptyState, MessageBarBody, Persona, PersonaConfig, PersonaSecondaryText,
};
use uf_product::primitives::{
    Button, ButtonAppearance, ButtonSize, DrawerBody, DrawerHeader, DrawerHeaderTitle,
    DrawerPosition, DrawerSize, Flex, FlexAlign, FlexGap, FlexJustify, MessageBar,
    MessageBarIntent, OverlayDrawer, Skeleton, SkeletonItem, Timestamp,
};

use crate::welcome::server::{list_featured_history, FeaturedChangeDto};

use super::AdminState;

#[derive(Clone)]
enum HistoryLoad {
    Loading,
    Ready(Vec<FeaturedChangeDto>),
    Failed,
}

/// Lists each change with who, what and when; Restore puts back the list from before it.
///
/// The drawer fetches its own history when opened, so opening it never reloads the page.
#[component]
pub(super) fn HistoryDrawer(state: AdminState, open: RwSignal<bool>) -> impl IntoView {
    let history = RwSignal::new(HistoryLoad::Loading);

    Effect::new(move |_| {
        state.history_reload.get();
        if !open.get() {
            return;
        }
        leptos::task::spawn_local_scoped(async move {
            match list_featured_history().await {
                Ok(rows) => history.set(HistoryLoad::Ready(rows)),
                Err(err) => {
                    state.report(&err, "load the history");
                    history.set(HistoryLoad::Failed);
                }
            }
        });
    });

    view! {
        <OverlayDrawer open=open position=DrawerPosition::Right size=DrawerSize::Medium close_on_esc=true>
            <DrawerHeader>
                <DrawerHeaderTitle>"Featured apps history"</DrawerHeaderTitle>
            </DrawerHeader>
            <DrawerBody>
                <div data-testid="featured-admin-history-drawer">
                    {move || match history.get() {
                        HistoryLoad::Loading => view! {
                            <Flex vertical=true gap=FlexGap::Medium>
                                {(0..3).map(|_| view! {
                                    <Skeleton><SkeletonItem width="100%" height="40px" /></Skeleton>
                                }).collect_view()}
                            </Flex>
                        }.into_any(),
                        HistoryLoad::Failed => view! {
                            <MessageBar intent=MessageBarIntent::Error>
                                <MessageBarBody>"Couldn't load the history. Close it and try again."</MessageBarBody>
                            </MessageBar>
                        }.into_any(),
                        HistoryLoad::Ready(rows) if rows.is_empty() => view! {
                            <EmptyState
                                message="No changes yet"
                                description="Each publish shows up here with who made it, so you can undo it."
                            />
                        }.into_any(),
                        HistoryLoad::Ready(rows) => view! {
                            <Flex vertical=true gap=FlexGap::Medium full_width=true>
                                {rows.into_iter().map(|change| view! {
                                    <HistoryItem state=state change=change />
                                }).collect_view()}
                            </Flex>
                        }.into_any(),
                    }}
                </div>
            </DrawerBody>
        </OverlayDrawer>
    }
}

#[component]
fn HistoryItem(state: AdminState, change: FeaturedChangeDto) -> impl IntoView {
    let change_id = StoredValue::new(change.id.clone());
    let restore_label = format!("Restore the list from before: {}", change.summary);
    view! {
        <Flex
            align=FlexAlign::Center
            justify=FlexJustify::SpaceBetween
            gap=FlexGap::Medium
            full_width=true
            attr:data-testid="featured-admin-history-item"
        >
            <Persona config=PersonaConfig::named(change.actor_name.clone())>
                <PersonaSecondaryText slot>{change.summary.clone()}</PersonaSecondaryText>
            </Persona>
            <Flex align=FlexAlign::Center gap=FlexGap::Small>
                <Caption1><Timestamp value=change.created_at.clone() /></Caption1>
                <Button
                    appearance=ButtonAppearance::Subtle
                    size=ButtonSize::Small
                    attr:data-testid="featured-admin-restore"
                    attr:aria-label=restore_label
                    on_click=Callback::new(move |_| {
                        state.restore(change_id.get_value(), "Featured apps restored.");
                    })
                >
                    "Restore"
                </Button>
            </Flex>
        </Flex>
    }
}
