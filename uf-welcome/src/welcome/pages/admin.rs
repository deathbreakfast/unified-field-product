//! Welcome featured-app admin page (`/welcome/admin`).

use leptos::prelude::*;
use leptos::tachys::view::any_view::IntoAny;
use leptos_router::components::A;
use uf_product::components::{Body1, ContentContainer, EmptyState, Field, Title3};
use uf_product::primitives::{
    Button, ButtonAppearance, Flex, FlexAlign, FlexGap, MessageBar, MessageBarIntent, Select,
    SelectBind,
};

use crate::welcome::server::{
    add_featured_app, can_manage_welcome_featured, get_featured_apps, list_manageable_apps,
    remove_featured_app, reorder_featured_apps, AppLinkDto, ManageableAppDto,
};

/// Admin UI for promoting apps onto the welcome Featured card.
#[component]
pub fn WelcomeAdminPage() -> impl IntoView {
    let can_manage = Resource::new(
        || (),
        |()| async move { can_manage_welcome_featured().await },
    );
    let featured = RwSignal::new(Vec::<AppLinkDto>::new());
    let catalog = RwSignal::new(Vec::<ManageableAppDto>::new());
    let selected = RwSignal::new(String::new());
    let error = RwSignal::new(Option::<String>::None);
    let reload = RwSignal::new(0u32);

    Effect::new(move |_| {
        reload.get();
        leptos::task::spawn_local_scoped(async move {
            match get_featured_apps().await {
                Ok(rows) => featured.set(rows),
                Err(e) => error.set(Some(e.to_string())),
            }
            if let Ok(apps) = list_manageable_apps().await {
                catalog.set(apps);
            }
        });
    });

    view! {
        <ContentContainer data_testid="welcome-admin-page">
            <Flex vertical=true gap=FlexGap::Large full_width=true>
                <Flex align=FlexAlign::Center gap=FlexGap::Medium full_width=true>
                    <Title3>"Featured apps"</Title3>
                    <A href="/welcome" attr:style="text-decoration: none; margin-left: auto;">
                        <Button appearance=ButtonAppearance::Secondary>"Back to welcome"</Button>
                    </A>
                </Flex>
                <Body1>"Add, remove, or reorder apps on the welcome Featured card. Visit stats live in Spectra."</Body1>

                {move || match can_manage.get() {
                    Some(Ok(false)) | Some(Err(_)) => view! {
                        <div data-testid="welcome-admin-denied">
                            <MessageBar intent=MessageBarIntent::Warning>
                                "WelcomeAdmin permission is required to manage featured apps."
                            </MessageBar>
                        </div>
                    }.into_any(),
                    _ => ().into_any(),
                }}

                {move || error.get().map(|msg| view! {
                    <MessageBar intent=MessageBarIntent::Error>{msg}</MessageBar>
                })}

                <div data-testid="featured-admin-list">
                    <Flex vertical=true gap=FlexGap::Medium full_width=true>
                    {move || {
                        let rows = featured.get();
                        if rows.is_empty() {
                            view! {
                                <EmptyState
                                    message="No featured apps"
                                    description="Pick an app below to promote it on welcome."
                                />
                            }.into_any()
                        } else {
                            let row_count = rows.len();
                            view! {
                                <Flex vertical=true gap=FlexGap::Small full_width=true>
                                    {rows.into_iter().enumerate().map(|(idx, app)| {
                                        let app_id = app.app_id.clone();
                                        let row_testid = format!("featured-admin-row-{}", app.app_id);
                                        let can_up = idx > 0;
                                        let can_down = idx + 1 < row_count;
                                        view! {
                                            <div data-testid=row_testid>
                                                <Flex
                                                    align=FlexAlign::Center
                                                    gap=FlexGap::Medium
                                                    full_width=true
                                                >
                                                    <Body1>{app.label.clone()}</Body1>
                                                    <Body1>{app.link.clone()}</Body1>
                                                    <div data-testid="featured-admin-move-up">
                                                        <Button
                                                            appearance=ButtonAppearance::Secondary
                                                            disabled=!can_up
                                                            on_click=Callback::new({
                                                                let app_id = app_id.clone();
                                                                move |_| {
                                                                    if !can_up {
                                                                        return;
                                                                    }
                                                                    let current = featured.get();
                                                                    let Some(pos) = current.iter().position(|a| a.app_id == app_id) else {
                                                                        return;
                                                                    };
                                                                    if pos == 0 {
                                                                        return;
                                                                    }
                                                                    let mut ids: Vec<String> =
                                                                        current.into_iter().map(|a| a.app_id).collect();
                                                                    ids.swap(pos, pos - 1);
                                                                    leptos::task::spawn_local_scoped(async move {
                                                                        match reorder_featured_apps(ids).await {
                                                                            Ok(()) => {
                                                                                error.set(None);
                                                                                reload.update(|n| *n += 1);
                                                                            }
                                                                            Err(e) => error.set(Some(e.to_string())),
                                                                        }
                                                                    });
                                                                }
                                                            })
                                                        >
                                                            "Move up"
                                                        </Button>
                                                    </div>
                                                    <div data-testid="featured-admin-move-down">
                                                        <Button
                                                            appearance=ButtonAppearance::Secondary
                                                            disabled=!can_down
                                                            on_click=Callback::new({
                                                                let app_id = app_id.clone();
                                                                move |_| {
                                                                    if !can_down {
                                                                        return;
                                                                    }
                                                                    let current = featured.get();
                                                                    let Some(pos) = current.iter().position(|a| a.app_id == app_id) else {
                                                                        return;
                                                                    };
                                                                    if pos + 1 >= current.len() {
                                                                        return;
                                                                    }
                                                                    let mut ids: Vec<String> =
                                                                        current.into_iter().map(|a| a.app_id).collect();
                                                                    ids.swap(pos, pos + 1);
                                                                    leptos::task::spawn_local_scoped(async move {
                                                                        match reorder_featured_apps(ids).await {
                                                                            Ok(()) => {
                                                                                error.set(None);
                                                                                reload.update(|n| *n += 1);
                                                                            }
                                                                            Err(e) => error.set(Some(e.to_string())),
                                                                        }
                                                                    });
                                                                }
                                                            })
                                                        >
                                                            "Move down"
                                                        </Button>
                                                    </div>
                                                    <div data-testid="featured-admin-remove">
                                                        <Button
                                                            appearance=ButtonAppearance::Secondary
                                                            on_click=Callback::new({
                                                                let app_id = app_id.clone();
                                                                move |_| {
                                                                    let id = app_id.clone();
                                                                    leptos::task::spawn_local_scoped(async move {
                                                                        match remove_featured_app(id).await {
                                                                            Ok(()) => {
                                                                                error.set(None);
                                                                                reload.update(|n| *n += 1);
                                                                            }
                                                                            Err(e) => error.set(Some(e.to_string())),
                                                                        }
                                                                    });
                                                                }
                                                            })
                                                        >
                                                            "Remove"
                                                        </Button>
                                                    </div>
                                                </Flex>
                                            </div>
                                        }
                                    }).collect_view()}
                                </Flex>
                            }.into_any()
                        }
                    }}
                    </Flex>
                </div>

                <Field label="Add app">
                    <Select bind=SelectBind::from(selected)>
                        <option value="">"Select an app"</option>
                        {move || {
                            let featured_ids: std::collections::HashSet<String> =
                                featured.get().into_iter().map(|a| a.app_id).collect();
                            catalog
                                .get()
                                .into_iter()
                                .filter(|a| !featured_ids.contains(&a.app_id))
                                .map(|a| {
                                    view! {
                                        <option value=a.app_id.clone()>{a.name}</option>
                                    }
                                })
                                .collect_view()
                        }}
                    </Select>
                </Field>
                <div data-testid="add-featured-app">
                    <Button
                        appearance=ButtonAppearance::Primary
                        on_click=Callback::new(move |_| {
                            let app_id = selected.get();
                            if app_id.is_empty() {
                                error.set(Some("Select an app first.".into()));
                                return;
                            }
                            let ordinal = featured.get().len() as i64;
                            leptos::task::spawn_local_scoped(async move {
                                match add_featured_app(app_id, ordinal).await {
                                    Ok(_) => {
                                        selected.set(String::new());
                                        error.set(None);
                                        reload.update(|n| *n += 1);
                                    }
                                    Err(e) => error.set(Some(e.to_string())),
                                }
                            });
                        })
                    >
                        "Add featured"
                    </Button>
                </div>
            </Flex>
        </ContentContainer>
    }
}
