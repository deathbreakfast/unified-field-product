//! AppBar workspace **content index** search (not picker [`crate::SearchSourcePicker`]).

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;
use orbital_primitives::{
    Combobox, ComboboxAppearance, ComboboxBind, ComboboxOption, Flex, FlexGap,
};
use uf_product::components::{Body1, Caption1, EmptyState};
use uf_product::primitives::{
    Button, ButtonAppearance, Dialog, DialogBody, DialogContent, DialogSurface, DialogTitle, List,
    OpenBind, SearchBox, SearchBoxAppearance, SearchBoxBind, Spinner, Tooltip,
};
use uf_product::use_auth_state;
use uf_product::workspace_search::{query_workspace_search, WorkspaceSearchHit};

fn safe_app_link(link: &str) -> Option<&str> {
    let t = link.trim();
    if t.starts_with('/') && !t.starts_with("//") && !t.contains(':') {
        Some(t)
    } else {
        None
    }
}

/// Desktop Combobox over the per-user workspace content index.
#[component]
pub fn WorkspaceSearch() -> impl IntoView {
    let auth = use_auth_state();
    let signed_in = Memo::new(move |_| auth.with(|s| s.is_authenticated()));
    let query = RwSignal::new(String::new());
    let selected = RwSignal::new(Vec::<String>::new());
    let hits = RwSignal::new(Vec::<WorkspaceSearchHit>::new());
    let navigate = use_navigate();
    let last_selected = RwSignal::new(None::<String>);

    let search_ready = StoredValue::new(false);
    Effect::new(move |_| {
        let q = query.get();
        if !search_ready.get_value() {
            search_ready.set_value(true);
            return;
        }
        if !signed_in.get() {
            hits.set(Vec::new());
            return;
        }
        let trimmed = q.trim().to_string();
        if trimmed.is_empty() {
            hits.set(Vec::new());
            return;
        }
        leptos::task::spawn_local_scoped(async move {
            match query_workspace_search(trimmed, 20).await {
                Ok(rows) => hits.set(rows),
                Err(_) => hits.set(Vec::new()),
            }
        });
    });

    Effect::new(move |_| {
        let keys = selected.get();
        let Some(key) = keys.last().cloned() else {
            return;
        };
        if last_selected.get_untracked().as_ref() == Some(&key) {
            return;
        }
        let rows = hits.get_untracked();
        if let Some(hit) = rows.into_iter().find(|h| h.id == key) {
            last_selected.set(Some(key));
            if let Some(path) = safe_app_link(&hit.link) {
                navigate(path, NavigateOptions::default());
            }
            query.set(String::new());
            selected.set(Vec::new());
            hits.set(Vec::new());
        }
    });

    let appearance = ComboboxAppearance {
        placeholder: MaybeProp::from(Signal::derive(move || {
            if signed_in.get() {
                "Search workspace…".to_string()
            } else {
                "Sign in to search".to_string()
            }
        })),
        disabled: Signal::derive(move || !signed_in.get()),
        clearable: true,
        ..Default::default()
    };

    view! {
        <div data-testid="app-bar-search-input" style="min-width: 250px; position: relative;">
            <Combobox
                bind=ComboboxBind::new(query, selected)
                appearance=appearance
            >
                <For
                    each=move || hits.get()
                    key=|h| h.id.clone()
                    let:hit
                >
                    {
                        let id = hit.id.clone();
                        let title = hit.title.clone();
                        let kind = hit.kind.clone();
                        let app_id = hit.app_id.clone();
                        view! {
                            <ComboboxOption text=title.clone() value=id>
                                <div data-testid="workspace-search-hit">
                                    <Flex vertical=true gap=FlexGap::Small>
                                        <Body1>{title}</Body1>
                                        <Caption1>{format!("{kind} • {app_id}")}</Caption1>
                                    </Flex>
                                </div>
                            </ComboboxOption>
                        }
                    }
                </For>
                <Show when=move || {
                    !query.get().trim().is_empty() && hits.get().is_empty()
                }>
                    <ComboboxOption text="No results" value="__empty__" disabled=Signal::from(true)>
                        <Caption1>"No matches"</Caption1>
                    </ComboboxOption>
                </Show>
            </Combobox>
        </div>
    }
}

/// Compact AppBar control: opens a Dialog with the same content-index search.
#[component]
pub fn WorkspaceSearchMobileTrigger() -> impl IntoView {
    let auth = use_auth_state();
    let signed_in = Memo::new(move |_| auth.with(|s| s.is_authenticated()));
    let open = RwSignal::new(false);
    view! {
        <Tooltip content=Signal::derive(move || {
            if signed_in.get() {
                "Search".to_string()
            } else {
                "Sign in to search".to_string()
            }
        })>
            <div data-testid="app-bar-search-mobile-trigger">
                <Button
                    appearance=ButtonAppearance::Subtle
                    icon=icondata::AiSearchOutlined
                    disabled=Signal::derive(move || !signed_in.get())
                    disabled_focusable=Signal::derive(move || !signed_in.get())
                    attr:aria-label="Search workspace"
                    on_click=Callback::new(move |_| {
                        if signed_in.get_untracked() {
                            open.set(true);
                        }
                    })
                />
            </div>
        </Tooltip>
        <WorkspaceSearchDialog open=open />
    }
}

/// Dialog body for compact workspace search.
#[component]
pub fn WorkspaceSearchDialog(
    /// Host-owned open binding.
    #[prop(into)]
    open: RwSignal<bool>,
) -> impl IntoView {
    let query = RwSignal::new(String::new());
    let navigate = use_navigate();
    let open_bind: OpenBind = open.into();

    Effect::new(move |_| {
        if !open.get() {
            query.set(String::new());
        }
    });

    let query_trimmed = Memo::new(move |_| query.get().trim().to_string());
    let hits_res = Resource::new(
        move || query_trimmed.get(),
        |q| async move {
            if q.is_empty() {
                return Ok::<Vec<WorkspaceSearchHit>, ServerFnError>(Vec::new());
            }
            query_workspace_search(q, 20).await
        },
    );

    let on_pick = Callback::new(move |link: String| {
        if let Some(path) = safe_app_link(&link).map(str::to_string) {
            open.set(false);
            navigate(&path, NavigateOptions::default());
        }
    });

    view! {
        <Dialog open=open_bind>
            <DialogSurface>
                <div data-testid="workspace-search-dialog">
                    <DialogBody>
                        <DialogTitle>"Search"</DialogTitle>
                        <DialogContent>
                            <Flex vertical=true gap=FlexGap::Medium full_width=true>
                                <div data-testid="workspace-search-dialog-input">
                                    <SearchBox
                                        bind=SearchBoxBind::from(query)
                                        appearance=SearchBoxAppearance::with_placeholder(
                                            "Search workspace…",
                                        )
                                    />
                                </div>
                                {move || {
                                    let q = query_trimmed.get();
                                    if q.is_empty() {
                                        return view! {
                                            <EmptyState
                                                message="Type to search"
                                                description="Results appear as you type."
                                                icon=icondata::AiSearchOutlined
                                            />
                                        }
                                        .into_any();
                                    }
                                    match hits_res.get() {
                                        None => view! { <Spinner /> }.into_any(),
                                        Some(Err(_)) => view! {
                                            <Caption1>"Couldn't search. Try again."</Caption1>
                                        }
                                        .into_any(),
                                        Some(Ok(rows)) if rows.is_empty() => view! {
                                            <EmptyState
                                                message="No matches"
                                                description="Try a different title."
                                                icon=icondata::AiSearchOutlined
                                            />
                                        }
                                        .into_any(),
                                        Some(Ok(rows)) => view! {
                                            <List>
                                                <For
                                                    each=move || rows.clone()
                                                    key=|h| h.id.clone()
                                                    let:hit
                                                >
                                                    {
                                                        let title = hit.title.clone();
                                                        let link = hit.link.clone();
                                                        let kind = hit.kind.clone();
                                                        view! {
                                                            <button
                                                                type="button"
                                                                data-testid="workspace-search-hit"
                                                                on:click=move |_| {
                                                                    on_pick.run(link.clone());
                                                                }
                                                            >
                                                                <Flex vertical=true gap=FlexGap::Small>
                                                                    <Body1>{title}</Body1>
                                                                    <Caption1>{kind}</Caption1>
                                                                </Flex>
                                                            </button>
                                                        }
                                                    }
                                                </For>
                                            </List>
                                        }
                                        .into_any(),
                                    }
                                }}
                            </Flex>
                        </DialogContent>
                    </DialogBody>
                </div>
            </DialogSurface>
        </Dialog>
    }
}
