//! Welcome featured-app admin page (`/welcome/admin`).
//!
//! Curators edit a local draft of the featured list, check the preview, and
//! publish it with a confirm. Members still see the published list until then.
//! Each publish is recorded, so History can restore an earlier list and the
//! toast after a publish can undo it. A draft started before someone else
//! published is refused rather than overwriting their change.
//!
//! Editing the draft only touches signals; the one fetch is keyed on a reload
//! counter, so moving or removing a row never reloads the page.

mod draft_bar;
mod history_drawer;
mod publish_dialog;
mod rows;

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use uf_product::components::{
    AutoGrid, Body1, Caption1, ContentContainer, EmptyState, EmptyStateCallToAction, Title2,
    ToastOptions,
};
use uf_product::primitives::{
    Breadcrumb, BreadcrumbButton, BreadcrumbDivider, BreadcrumbItem, Button, ButtonAppearance,
    Flex, FlexAlign, FlexGap, Link, MessageBar, MessageBarIntent, Skeleton, SkeletonItem,
    Timestamp, ToastIntent, ToasterInjection,
};
use uf_product::services::{parse_permission_server_error, PermissionServerError};

use crate::welcome::server::{
    get_featured_admin, restore_featured, FeaturedAdminAppDto, FeaturedChangeDto,
};

use draft_bar::DraftBar;
use history_drawer::HistoryDrawer;
use publish_dialog::PublishDialog;
use rows::{AddAppField, FeaturedPreview, FeaturedRows};

/// How far the first load got.
#[derive(Debug, Clone, PartialEq, Eq)]
enum LoadState {
    Loading,
    Ready,
    Denied,
    Failed,
}

/// Page state shared by the admin sub-components.
#[derive(Clone, Copy)]
pub(super) struct AdminState {
    /// Published list as last loaded (or as just published).
    published: RwSignal<Vec<FeaturedAdminAppDto>>,
    /// Every registered app, for names, descriptions and the Add picker.
    catalog: RwSignal<Vec<FeaturedAdminAppDto>>,
    /// Draft list of app ids, in order.
    draft: RwSignal<Vec<String>>,
    /// Version the draft is based on.
    version: RwSignal<Option<String>>,
    /// Newest change, for "Last published by".
    last_change: RwSignal<Option<FeaturedChangeDto>>,
    /// Who published while this draft was open.
    conflict: RwSignal<Option<String>>,
    /// Failed action, in words.
    action_error: RwSignal<Option<String>>,
    /// First-load outcome.
    load: RwSignal<LoadState>,
    /// Replace the draft with the published list on the next load.
    reset_draft: RwSignal<bool>,
    /// Bumped to refetch the published list.
    reload: RwSignal<u32>,
    /// Bumped to refetch History when it is open.
    history_reload: RwSignal<u32>,
}

impl AdminState {
    fn new() -> Self {
        Self {
            published: RwSignal::new(Vec::new()),
            catalog: RwSignal::new(Vec::new()),
            draft: RwSignal::new(Vec::new()),
            version: RwSignal::new(None),
            last_change: RwSignal::new(None),
            conflict: RwSignal::new(None),
            action_error: RwSignal::new(None),
            load: RwSignal::new(LoadState::Loading),
            reset_draft: RwSignal::new(true),
            reload: RwSignal::new(0),
            history_reload: RwSignal::new(0),
        }
    }

    /// Published app ids, in order.
    fn published_ids(&self) -> Vec<String> {
        self.published
            .with(|apps| apps.iter().map(|a| a.app_id.clone()).collect())
    }

    /// Whether the draft differs from the published list.
    fn is_dirty(&self) -> bool {
        let published = self.published_ids();
        self.draft.with(|draft| *draft != published)
    }

    /// Display name for an app id.
    fn app_name(&self, app_id: &str) -> String {
        self.catalog.with(|apps| {
            apps.iter()
                .find(|a| a.app_id == app_id)
                .map_or_else(|| app_id.to_string(), |a| a.name.clone())
        })
    }

    /// Registration details for an app id.
    fn app(&self, app_id: &str) -> Option<FeaturedAdminAppDto> {
        self.catalog
            .with(|apps| apps.iter().find(|a| a.app_id == app_id).cloned())
    }

    /// Refetch the published list and replace the draft with it.
    fn reload_fresh(&self) {
        self.reset_draft.set(true);
        self.reload.update(|n| *n += 1);
        self.history_reload.update(|n| *n += 1);
    }

    /// Turn a refused or failed call into words. A refused call switches the page
    /// to the denied state instead, so the permission payload never shows.
    fn report(&self, err: &ServerFnError, what: &str) {
        if matches!(
            parse_permission_server_error(&err.to_string()),
            Some(PermissionServerError::Denied { .. })
        ) {
            self.load.set(LoadState::Denied);
            return;
        }
        self.action_error.set(Some(format!(
            "Couldn't {what}. Check your connection and try again."
        )));
    }

    /// Restore the list from before `change_id`, then offer to undo the restore.
    fn restore(self, change_id: String, done_title: &'static str) {
        leptos::task::spawn_local_scoped(async move {
            match restore_featured(change_id).await {
                Ok(change) => {
                    self.action_error.set(None);
                    self.conflict.set(None);
                    self.reload_fresh();
                    self.toast(done_title, change.id);
                }
                Err(err) => self.report(&err, "restore that list"),
            }
        });
    }

    /// Success toast with an Undo that restores the list from before `change_id`.
    fn toast(self, title: &'static str, change_id: String) {
        let Some(toaster) = use_context::<ToasterInjection>() else {
            return;
        };
        let undo = Callback::new(move |()| {
            self.restore(change_id.clone(), "Change undone.");
        });
        toaster.dispatch(
            ToastOptions::new(title)
                .intent(ToastIntent::Success.into())
                .footer_action_callback("Undo", true, Some(undo)),
        );
    }
}

/// Admin UI for curating the welcome Featured card.
#[component]
pub fn WelcomeAdminPage() -> impl IntoView {
    let state = AdminState::new();
    let history_open = RwSignal::new(false);
    let publish_open = RwSignal::new(false);

    Effect::new(move |_| {
        state.reload.get();
        leptos::task::spawn_local_scoped(async move {
            match get_featured_admin().await {
                Ok(dto) => {
                    let keep_draft = !state.reset_draft.get_untracked()
                        && state.load.get_untracked() == LoadState::Ready
                        && untrack(|| state.is_dirty());
                    let ids: Vec<String> = dto.published.iter().map(|a| a.app_id.clone()).collect();
                    state.catalog.set(dto.catalog);
                    state.published.set(dto.published);
                    state.version.set(dto.version);
                    state.last_change.set(dto.last_change);
                    if !keep_draft {
                        state.draft.set(ids);
                    }
                    state.reset_draft.set(false);
                    state.conflict.set(None);
                    state.load.set(LoadState::Ready);
                }
                Err(err) => {
                    if state.load.get_untracked() == LoadState::Ready {
                        state.report(&err, "load the featured apps");
                    } else if matches!(
                        parse_permission_server_error(&err.to_string()),
                        Some(PermissionServerError::Denied { .. })
                    ) {
                        state.load.set(LoadState::Denied);
                    } else {
                        state.load.set(LoadState::Failed);
                    }
                }
            }
        });
    });

    let load = Memo::new(move |_| state.load.get());

    view! {
        <ContentContainer data_testid="welcome-admin-page">
            <Flex vertical=true gap=FlexGap::Large full_width=true>
                <Flex vertical=true gap=FlexGap::Small full_width=true>
                    <Breadcrumb>
                        <BreadcrumbItem>
                            <BreadcrumbButton href="/welcome".to_string()>"Welcome"</BreadcrumbButton>
                        </BreadcrumbItem>
                        <BreadcrumbDivider />
                        <BreadcrumbItem>
                            <BreadcrumbButton current=true>"Featured apps"</BreadcrumbButton>
                        </BreadcrumbItem>
                    </Breadcrumb>
                    <Title2>"Featured apps"</Title2>
                    {move || (load.get() != LoadState::Denied).then(|| view! {
                        <Body1>"Every member sees these on Welcome, in this order. New members see them first."</Body1>
                        <LastPublished state=state history_open=history_open />
                    })}
                </Flex>
                {move || match load.get() {
                    LoadState::Denied => view! { <DeniedState /> }.into_any(),
                    LoadState::Failed => view! {
                        <div data-testid="featured-admin-error">
                            <MessageBar intent=MessageBarIntent::Error>
                                <uf_product::components::MessageBarBody>
                                    "Couldn't load the featured apps. Check your connection and try again."
                                </uf_product::components::MessageBarBody>
                                <uf_product::components::MessageBarActions>
                                    <Button
                                        appearance=ButtonAppearance::Secondary
                                        attr:data-testid="featured-admin-retry"
                                        on_click=Callback::new(move |_| {
                                            state.load.set(LoadState::Loading);
                                            state.reload_fresh();
                                        })
                                    >
                                        "Try again"
                                    </Button>
                                </uf_product::components::MessageBarActions>
                            </MessageBar>
                        </div>
                    }.into_any(),
                    LoadState::Loading | LoadState::Ready => view! {
                        <Editor state=state loading=Signal::derive(move || load.get() == LoadState::Loading) publish_open=publish_open />
                    }.into_any(),
                }}
            </Flex>
            <PublishDialog state=state open=publish_open />
            <HistoryDrawer state=state open=history_open />
        </ContentContainer>
    }
}

/// "Last published by {name} · {time}" with the History link.
#[component]
fn LastPublished(state: AdminState, history_open: RwSignal<bool>) -> impl IntoView {
    view! {
        <Flex align=FlexAlign::Center gap=FlexGap::Small attr:data-testid="featured-admin-last-published">
            {move || state.last_change.get().map_or_else(
                || view! { <Caption1>"Not published yet"</Caption1> }.into_any(),
                |change| view! {
                    <Caption1>{format!("Last published by {} ·", change.actor_name)}</Caption1>
                    <Caption1><Timestamp value=change.created_at.clone() /></Caption1>
                }.into_any(),
            )}
            <Link inline=true attr:data-testid="featured-admin-history-link" on:click=move |_| history_open.set(true)>
                "History"
            </Link>
        </Flex>
    }
}

/// Shown instead of every control when the member lacks `WelcomeAdmin`.
#[component]
fn DeniedState() -> impl IntoView {
    let back = use_navigate();
    let ask = use_navigate();
    view! {
        <div data-testid="welcome-admin-denied">
            <EmptyState
                message="Only site admins can change featured apps"
                description="You need the Welcome admin permission. Featured apps on Welcome are the same for everyone."
            >
                <EmptyStateCallToAction slot:call_to_action>
                    <Flex gap=FlexGap::Small>
                        <Button
                            appearance=ButtonAppearance::Primary
                            attr:data-testid="featured-admin-back"
                            on_click=Callback::new(move |_| back("/welcome", Default::default()))
                        >
                            "Back to Welcome"
                        </Button>
                        // The shell's Request Permission dialog isn't wired yet; this is the
                        // same destination the shell's own denied gate uses.
                        <Button
                            appearance=ButtonAppearance::Secondary
                            attr:data-testid="featured-admin-ask-access"
                            on_click=Callback::new(move |_| {
                                ask(uf_product::paths::PERMISSION_PERMISSIONS, Default::default());
                            })
                        >
                            "Ask for access"
                        </Button>
                    </Flex>
                </EmptyStateCallToAction>
            </EmptyState>
        </div>
    }
}

/// Draft bar, messages, list, Add picker and preview.
#[component]
fn Editor(state: AdminState, loading: Signal<bool>, publish_open: RwSignal<bool>) -> impl IntoView {
    view! {
        {move || state.conflict.get().map(|name| view! {
            <div data-testid="featured-admin-conflict">
                <MessageBar intent=MessageBarIntent::Error>
                    <uf_product::components::MessageBarBody>
                        {format!("{name} published a change while you were editing. Reload to see it; your draft is kept.")}
                    </uf_product::components::MessageBarBody>
                    <uf_product::components::MessageBarActions>
                        <Button
                            appearance=ButtonAppearance::Secondary
                            attr:data-testid="featured-admin-reload"
                            on_click=Callback::new(move |_| state.reload.update(|n| *n += 1))
                        >
                            "Reload"
                        </Button>
                    </uf_product::components::MessageBarActions>
                </MessageBar>
            </div>
        })}
        {move || state.action_error.get().map(|msg| view! {
            <div data-testid="featured-admin-action-error">
                <MessageBar intent=MessageBarIntent::Error>
                    <uf_product::components::MessageBarBody>{msg}</uf_product::components::MessageBarBody>
                    <uf_product::components::MessageBarActions>
                        <Button
                            appearance=ButtonAppearance::Secondary
                            on_click=Callback::new(move |_| {
                                state.action_error.set(None);
                                state.reload.update(|n| *n += 1);
                            })
                        >
                            "Try again"
                        </Button>
                    </uf_product::components::MessageBarActions>
                </MessageBar>
            </div>
        })}
        <DraftBar state=state publish_open=publish_open />
        <AutoGrid min="320px">
            <rows::ListCard>
                {move || if loading.get() {
                    view! { <LoadingRows /> }.into_any()
                } else {
                    view! {
                        <FeaturedRows state=state />
                        <AddAppField state=state />
                    }.into_any()
                }}
            </rows::ListCard>
            <FeaturedPreview state=state loading=loading />
        </AutoGrid>
    }
}

/// Placeholder rows sized like the real ones while the list loads.
#[component]
fn LoadingRows() -> impl IntoView {
    view! {
        <div data-testid="featured-admin-loading">
            <Flex vertical=true gap=FlexGap::Medium full_width=true>
                {(0..3).map(|_| view! {
                    <Flex align=FlexAlign::Center gap=FlexGap::Medium full_width=true>
                        <Skeleton><SkeletonItem width="32px" height="32px" /></Skeleton>
                        <Flex vertical=true gap=FlexGap::Small full_width=true>
                            <Skeleton><SkeletonItem width="40%" height="14px" /></Skeleton>
                            <Skeleton><SkeletonItem width="70%" height="12px" /></Skeleton>
                        </Flex>
                    </Flex>
                }).collect_view()}
            </Flex>
        </div>
    }
}
