//! Draft rows, the Add picker and the members' preview.

use leptos::prelude::*;
use uf_product::components::{
    AutoGrid, Body1, Caption1, Card, CardContent, CardHeader, EmptyState, Subtitle2,
};
use uf_product::primitives::{
    Avatar, AvatarColor, AvatarConfig, AvatarShape, Badge, BadgeAppearance, Button,
    ButtonAppearance, Combobox, ComboboxAppearance, ComboboxBind, ComboboxOption, Field, Flex,
    FlexAlign, FlexGap, Skeleton, SkeletonItem, Table, TableBody, TableCell, TableRow,
};

use super::AdminState;

/// Square app icon: the app's initial on a colour picked from its id.
fn app_avatar(app_id: &str, name: &str, size: u8) -> impl IntoView {
    view! {
        <Avatar config=AvatarConfig {
            name: Some(name.to_string()),
            shape: AvatarShape::Square,
            size: Some(size),
            color: AvatarColor::Colorful,
            id_for_color: Some(app_id.to_string()),
            ..Default::default()
        } />
    }
}

/// "In this order" card around the rows and the Add picker.
#[component]
pub(super) fn ListCard(children: Children) -> impl IntoView {
    view! {
        <Card>
            <CardHeader>
                <Subtitle2>"In this order"</Subtitle2>
            </CardHeader>
            <CardContent>
                <Flex vertical=true gap=FlexGap::Medium full_width=true>
                    {children()}
                </Flex>
            </CardContent>
        </Card>
    }
}

/// One table row per draft app, with Move up, Move down and Remove.
#[component]
pub(super) fn FeaturedRows(state: AdminState) -> impl IntoView {
    let is_empty = Memo::new(move |_| state.draft.with(Vec::is_empty));
    view! {
        <div data-testid="featured-admin-list">
            <Show
                when=move || !is_empty.get()
                fallback=|| view! {
                    <div data-testid="featured-admin-empty">
                        <EmptyState
                            message="Nothing featured"
                            description="Welcome hides the Featured card until you add an app. Pick two or three that a new member should open first."
                        />
                    </div>
                }
            >
                <Table>
                    <TableBody>
                        <For
                            each=move || state.draft.get()
                            key=|app_id| app_id.clone()
                            let:app_id
                        >
                            <FeaturedRow state=state app_id=app_id />
                        </For>
                    </TableBody>
                </Table>
            </Show>
        </div>
    }
}

/// Move `app_id` by `delta` places in the draft.
fn move_in_draft(state: AdminState, app_id: &str, delta: isize) {
    state.draft.update(|draft| {
        let Some(pos) = draft.iter().position(|id| id == app_id) else {
            return;
        };
        let Some(target) = pos.checked_add_signed(delta).filter(|t| *t < draft.len()) else {
            return;
        };
        draft.swap(pos, target);
    });
}

#[component]
fn FeaturedRow(state: AdminState, app_id: String) -> impl IntoView {
    let app = state.app(&app_id);
    let name = app
        .as_ref()
        .map_or_else(|| app_id.clone(), |a| a.name.clone());
    let description = app.map(|a| a.description).unwrap_or_default();
    let id = StoredValue::new(app_id.clone());
    let position = Memo::new(move |_| {
        id.with_value(|id| state.draft.with(|d| d.iter().position(|x| x == id)))
    });
    let is_first = Signal::derive(move || position.get() == Some(0));
    let is_last = Signal::derive(move || {
        let len = state.draft.with(Vec::len);
        position.get().is_none_or(|p| p + 1 >= len)
    });
    let is_new = Signal::derive(move || {
        id.with_value(|id| state.published.with(|p| !p.iter().any(|a| &a.app_id == id)))
    });
    let row_testid = format!("featured-admin-row-{app_id}");
    let avatar = app_avatar(&app_id, &name, 32);
    let up_label = format!("Move {name} up");
    let down_label = format!("Move {name} down");
    let remove_label = format!("Remove {name}");

    view! {
        <TableRow attr:data-testid=row_testid>
            <TableCell>
                <Flex align=FlexAlign::Center gap=FlexGap::Medium>
                    {avatar}
                    <Flex vertical=true gap=FlexGap::Size(2)>
                        <Flex align=FlexAlign::Center gap=FlexGap::Small>
                            <Body1>{name}</Body1>
                            <Show when=move || is_new.get()>
                                <Badge appearance=BadgeAppearance::Tint>"Added"</Badge>
                            </Show>
                        </Flex>
                        <Caption1>{description}</Caption1>
                    </Flex>
                </Flex>
            </TableCell>
            <TableCell>
                <Flex gap=FlexGap::Small>
                    <Button
                        appearance=ButtonAppearance::Subtle
                        icon=icondata::AiArrowUpOutlined
                        disabled=is_first
                        attr:aria-label=up_label
                        attr:data-testid="featured-admin-move-up"
                        on_click=Callback::new(move |_| id.with_value(|id| move_in_draft(state, id, -1)))
                    />
                    <Button
                        appearance=ButtonAppearance::Subtle
                        icon=icondata::AiArrowDownOutlined
                        disabled=is_last
                        attr:aria-label=down_label
                        attr:data-testid="featured-admin-move-down"
                        on_click=Callback::new(move |_| id.with_value(|id| move_in_draft(state, id, 1)))
                    />
                    <Button
                        appearance=ButtonAppearance::Subtle
                        icon=icondata::AiDeleteOutlined
                        attr:aria-label=remove_label
                        attr:data-testid="featured-admin-remove"
                        on_click=Callback::new(move |_| {
                            id.with_value(|id| state.draft.update(|d| d.retain(|x| x != id)));
                        })
                    />
                </Flex>
            </TableCell>
        </TableRow>
    }
}

/// "Add an app" picker over registered apps that aren't in the draft yet.
#[component]
pub(super) fn AddAppField(state: AdminState) -> impl IntoView {
    let query = RwSignal::new(String::new());
    let selected = RwSignal::new(Vec::<String>::new());
    let options = Memo::new(move |_| {
        state.draft.with(|draft| {
            state.catalog.with(|apps| {
                apps.iter()
                    .filter(|a| !draft.contains(&a.app_id))
                    .cloned()
                    .collect::<Vec<_>>()
            })
        })
    });
    let appearance = ComboboxAppearance {
        placeholder: MaybeProp::from("Search registered apps".to_string()),
        ..Default::default()
    };

    view! {
        <Field label="Add an app">
            <Flex align=FlexAlign::Center gap=FlexGap::Small full_width=true>
                <div data-testid="featured-admin-add-picker">
                    <Combobox bind=ComboboxBind::new(query, selected) appearance=appearance>
                        <For
                            each=move || options.get()
                            key=|app| app.app_id.clone()
                            let:app
                        >
                            <ComboboxOption text=app.name.clone() value=app.app_id.clone() />
                        </For>
                    </Combobox>
                </div>
                <Button
                    appearance=ButtonAppearance::Secondary
                    attr:data-testid="add-featured-app"
                    disabled=Signal::derive(move || selected.with(Vec::is_empty))
                    on_click=Callback::new(move |_| {
                        let Some(app_id) = selected.get_untracked().into_iter().next() else {
                            return;
                        };
                        state.draft.update(|d| {
                            if !d.contains(&app_id) {
                                d.push(app_id);
                            }
                        });
                        selected.set(Vec::new());
                        query.set(String::new());
                    })
                >
                    "Add"
                </Button>
            </Flex>
        </Field>
    }
}

/// The tiles members will see on Welcome once the draft is published.
#[component]
pub(super) fn FeaturedPreview(state: AdminState, loading: Signal<bool>) -> impl IntoView {
    view! {
        <Card attr:data-testid="featured-admin-preview">
            <CardHeader>
                <Subtitle2>"Preview"</Subtitle2>
            </CardHeader>
            <CardContent>
                {move || {
                    if loading.get() {
                        return view! {
                            <Skeleton><SkeletonItem width="100%" height="64px" /></Skeleton>
                        }.into_any();
                    }
                    if state.draft.with(Vec::is_empty) {
                        return view! {
                            <Body1>"The card is hidden while nothing is featured."</Body1>
                        }.into_any();
                    }
                    view! {
                        <AutoGrid min="200px">
                            <For
                                each=move || state.draft.get()
                                key=|app_id| app_id.clone()
                                let:app_id
                            >
                                <PreviewTile state=state app_id=app_id />
                            </For>
                        </AutoGrid>
                    }.into_any()
                }}
            </CardContent>
        </Card>
    }
}

#[component]
fn PreviewTile(state: AdminState, app_id: String) -> impl IntoView {
    let app = state.app(&app_id);
    let name = app
        .as_ref()
        .map_or_else(|| app_id.clone(), |a| a.name.clone());
    let description = app.map(|a| a.description).unwrap_or_default();
    let testid = format!("featured-admin-preview-tile-{app_id}");
    let avatar = app_avatar(&app_id, &name, 40);
    view! {
        <Card attr:data-testid=testid>
            <CardContent>
                <Flex align=FlexAlign::Center gap=FlexGap::Medium>
                    {avatar}
                    <Flex vertical=true gap=FlexGap::Size(2)>
                        <Subtitle2>{name}</Subtitle2>
                        <Caption1>{description}</Caption1>
                    </Flex>
                </Flex>
            </CardContent>
        </Card>
    }
}
