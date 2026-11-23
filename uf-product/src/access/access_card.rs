//! [`AccessCard`]: who can act on one resource, and, for owners, the controls to change it.

use leptos::prelude::*;

use super::add_access_dialog::{role_options, AddAccessDialog};
use super::parts::{join_names, principal_persona, tint_badge};
use super::remove_access_dialog::{write_error_text, RemoveAccessDialog};
use super::server::{load_resource_access, set_resource_access_role};
use super::step_up::spawn_with_step_up;
use super::{AccessEntry, AccessLoad, AccessView, PrincipalKind, PrincipalRef, ResourceRef, Role};
use crate::components::{
    Body1, Caption2, Card, EmptyState, SelectOption, SkeletonItemSize, SpacingSize, Subtitle1,
};
use crate::primitives::{
    use_breakpoint_down, Breakpoint, Button, ButtonAppearance, Divider, Flex, FlexAlign, FlexGap,
    FlexJustify, FlexWrap, Menu, MenuItem, MenuTrigger, MessageBar, MessageBarIntent,
    MessageBarLayout, Select, SelectAppearance, SelectSize, SkeletonItem,
};

/// What the card is showing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CardState {
    /// The access read is running.
    Loading,
    /// The access read failed; carries the server's message.
    Error(String),
    /// The host wired no backend.
    Unwired,
    /// The rows, as the reader sees them.
    Ready(AccessView),
}

impl CardState {
    /// The state for a finished load.
    #[must_use]
    pub fn from_load(load: Result<AccessLoad, ServerFnError>) -> Self {
        match load {
            Ok(AccessLoad::Ready(view)) => Self::Ready(view),
            Ok(AccessLoad::Unwired) => Self::Unwired,
            Err(e) => Self::Error(e.to_string()),
        }
    }
}

/// What the card's controls do. Copy, so every row can hold it.
#[derive(Clone, Copy)]
pub(crate) struct CardActions {
    /// Load the rows again.
    pub retry: Callback<()>,
    /// Open Add people.
    pub add: Callback<()>,
    /// Change a row's role.
    pub set_role: Callback<(PrincipalRef, Role)>,
    /// Ask to remove a row.
    pub remove: Callback<PrincipalRef>,
}

fn header(noun: &str, add: Option<Callback<()>>) -> impl IntoView {
    let caption = format!("Who can view, run, edit and own this {noun}.");
    let add = add.map(|add| {
        view! {
            <Button
                appearance=ButtonAppearance::Secondary
                on_click=Callback::new(move |_| add.run(()))
                attr:data-testid="access-card-add"
            >
                "Add people"
            </Button>
        }
    });
    view! {
        <Flex
            justify=FlexJustify::SpaceBetween
            align=FlexAlign::Start
            gap=FlexGap::Small
            wrap=FlexWrap::Wrap
            full_width=true
        >
            <Flex vertical=true gap=FlexGap::Small>
                <Subtitle1>"Access"</Subtitle1>
                <Body1>{caption}</Body1>
            </Flex>
            {add}
        </Flex>
    }
}

fn row_badges(entry: &AccessEntry) -> impl IntoView {
    let team = matches!(entry.principal.kind, PrincipalKind::Team { .. })
        .then(|| tint_badge("Team", "access-row-team"));
    let default_owner = entry
        .default_owner
        .then(|| tint_badge("Default owner", "access-row-default-owner"));
    let umbrella = entry
        .umbrella
        .then(|| tint_badge("Kind-wide", "access-row-kind-wide"));
    view! { {team} {default_owner} {umbrella} }
}

/// A row's role Select and its Remove menu.
fn owner_controls(entry: &AccessEntry, roles: &[Role], actions: CardActions) -> impl IntoView {
    let initial = entry.role.key();
    let role_key = RwSignal::new(initial.to_string());
    let mut options = role_options(roles)
        .into_iter()
        .map(|role| view! { <SelectOption value=role.key() label=role.label() /> }.into_any())
        .collect::<Vec<_>>();
    if let Role::Custom(_) = entry.role {
        options.push(
            view! {
                <SelectOption
                    value=entry.role.key()
                    label=entry.role.label()
                    disabled=Signal::from(true)
                />
            }
            .into_any(),
        );
    }
    let principal = entry.principal.clone();
    Effect::new({
        let principal = principal.clone();
        move |_| {
            let key = role_key.get();
            if key == initial {
                return;
            }
            if let Some(role) = Role::from_key(&key) {
                actions.set_role.run((principal.clone(), role));
            }
        }
    });
    let name = principal.label.clone();
    view! {
        <Flex gap=FlexGap::Small align=FlexAlign::Center>
            <span data-testid="access-row-role-select">
                <Select
                    bind=role_key
                    appearance=SelectAppearance {
                        size: Signal::from(SelectSize::Small),
                        ..Default::default()
                    }
                >
                    {options}
                </Select>
            </span>
            <Menu on_select=move |action: &str| {
                if action == "remove" {
                    actions.remove.run(principal.clone());
                }
            }>
                <MenuTrigger slot>
                    <Button
                        appearance=ButtonAppearance::Subtle
                        icon=icondata::AiMoreOutlined
                        attr:aria-label=format!("Actions for {name}")
                        attr:data-testid="access-row-menu"
                    />
                </MenuTrigger>
                <MenuItem value="remove">"Remove"</MenuItem>
            </Menu>
        </Flex>
    }
}

fn row(
    entry: &AccessEntry,
    view: &AccessView,
    stacked: bool,
    actions: CardActions,
) -> impl IntoView {
    let right = if view.can_maintain {
        owner_controls(entry, &view.roles, actions).into_any()
    } else {
        let label = entry.role.label();
        view! {
            <span data-testid="access-row-role">
                <Body1>{label}</Body1>
            </span>
        }
        .into_any()
    };
    let align = if stacked {
        FlexAlign::Start
    } else {
        FlexAlign::Center
    };
    let key = entry.principal.key();
    let role_key = entry.role.key();
    let persona = principal_persona(&entry.principal);
    let badges = row_badges(entry).into_any();
    view! {
        <Flex
            vertical=stacked
            justify=FlexJustify::SpaceBetween
            align=align
            gap=FlexGap::Small
            full_width=true
            attr:data-testid="access-row"
            attr:data-key=key
            attr:data-role=role_key
        >
            <Flex gap=FlexGap::Small align=FlexAlign::Center wrap=FlexWrap::Wrap>
                {persona}
                {badges}
            </Flex>
            {right}
        </Flex>
        <Divider />
    }
}

/// The info bar a reader sees: their own role, then who to ask.
#[must_use]
pub fn reader_notice(view: &AccessView, noun: &str) -> String {
    let owners = view.owner_names();
    let ask = (!owners.is_empty()).then(|| {
        format!(
            "To change who has access, ask an owner: {}.",
            join_names(&owners)
        )
    });
    match (view.viewer_role.as_ref(), ask) {
        (Some(role), Some(ask)) => format!("{} {ask}", role.you_can(noun)),
        (Some(role), None) => role.you_can(noun),
        (None, Some(ask)) => ask,
        (None, None) => "Only owners can change who has access.".to_string(),
    }
}

fn ready_body(
    resource: &ResourceRef,
    view: &AccessView,
    stacked: bool,
    actions: CardActions,
) -> AnyView {
    let noun = &resource.noun;
    let notice = (!view.can_maintain).then(|| {
        let text = reader_notice(view, noun);
        view! {
            <span data-testid="access-card-reader-notice">
                <MessageBar intent=MessageBarIntent::Info>{text}</MessageBar>
            </span>
        }
    });
    let rows = view
        .entries
        .iter()
        .map(|entry| row(entry, view, stacked, actions))
        .collect_view();
    let restricted = view.restricted().then(|| {
        let text = format!("Only the people and groups above can see this {noun}.");
        view! {
            <span data-testid="access-card-restricted">
                <Caption2>{text}</Caption2>
            </span>
        }
    });
    let super_user = format!("Super User can always manage every {noun}.");
    view! {
        {header(noun, view.can_maintain.then_some(actions.add))}
        {notice}
        <Flex vertical=true gap=FlexGap::Small full_width=true attr:data-testid="access-card-rows">
            {rows}
        </Flex>
        {restricted}
        <span data-testid="access-card-super-user">
            <Caption2>{super_user}</Caption2>
        </span>
    }
    .into_any()
}

/// The card's content for one state. Pure, so each state can be rendered alone.
pub(crate) fn card_body(
    resource: &ResourceRef,
    state: CardState,
    stacked: bool,
    actions: CardActions,
) -> AnyView {
    let noun = &resource.noun;
    match state {
        CardState::Loading => view! {
            {header(noun, None)}
            <Flex vertical=true gap=FlexGap::Small full_width=true attr:data-testid="access-card-loading">
                {(0..3).map(|_| view! {
                    <Flex gap=FlexGap::Small align=FlexAlign::Center full_width=true>
                        <SkeletonItem size=Signal::from(SkeletonItemSize::S32) width="32px".to_string() />
                        <SkeletonItem size=Signal::from(SkeletonItemSize::S16) width="40%".to_string() />
                    </Flex>
                }).collect_view()}
            </Flex>
        }
        .into_any(),
        CardState::Error(message) => {
            let text = format!("Couldn't load who has access: {message}");
            view! {
                {header(noun, None)}
                <span data-testid="access-card-error">
                    <MessageBar intent=MessageBarIntent::Error layout=MessageBarLayout::Multiline>
                        <Flex vertical=true gap=FlexGap::Small>
                            <Body1>{text}</Body1>
                            <Button
                                appearance=ButtonAppearance::Secondary
                                on_click=Callback::new(move |_| actions.retry.run(()))
                                attr:data-testid="access-card-retry"
                            >
                                "Retry"
                            </Button>
                        </Flex>
                    </MessageBar>
                </span>
            }
            .into_any()
        }
        CardState::Unwired => view! {
            {header(noun, None)}
            <div data-testid="access-card-unwired">
                <EmptyState
                    message="Access control isn't set up on this host."
                    description="Ask whoever runs this deployment to set it up. Until then nobody can act on this."
                    icon=icondata::AiLockOutlined
                />
            </div>
        }
        .into_any(),
        CardState::Ready(view) => ready_body(resource, &view, stacked, actions),
    }
}

/// Who can view, run, edit and own one resource.
///
/// Owners get a role Select and a Remove menu on each row, and Add people. Everyone else sees
/// the rows read-only, with their own role and who to ask in an info bar. Team groups carry a
/// Team badge and their size; the app's admin group is tagged Default owner and kind-wide
/// groups Kind-wide. With no kind-wide row left, the card says only the listed rows can see
/// the resource. Rows load as skeletons, a failed load keeps its message and offers Retry,
/// and a host with no backend says access control isn't set up. Below the Md breakpoint each
/// row stacks.
///
/// Role changes and adds ask for a recent authenticator code when the server wants one.
///
/// ```rust,ignore
/// use leptos::prelude::*;
/// use uf_product::access::{AccessCard, ResourceRef};
///
/// let job = ResourceRef::new("chronon_job", "billing-rollup")
///     .label("Billing rollup")
///     .noun("job");
/// view! { <AccessCard resource=job /> };
/// ```
#[component]
pub fn AccessCard(
    /// The resource whose access the card shows.
    resource: ResourceRef,
) -> impl IntoView {
    let dialog_resource = resource.clone();
    let resource = StoredValue::new(resource);
    let reload = RwSignal::new(0_u32);
    let load = Resource::new(
        move || (resource.get_value(), reload.get()),
        |(r, _)| load_resource_access(r),
    );
    let stacked = use_breakpoint_down(Breakpoint::Md);
    let notice = RwSignal::new(None::<String>);
    let error = RwSignal::new(None::<String>);
    let add_open = RwSignal::new(false);
    let remove_open = RwSignal::new(false);
    let removing = RwSignal::new(None::<PrincipalRef>);
    let latest = RwSignal::new(None::<AccessView>);

    Effect::new(move |_| {
        if let Some(Ok(AccessLoad::Ready(view))) = load.get() {
            latest.set(Some(view));
        }
    });
    // A refused change reloads, so the row's Select goes back to the role it holds.
    Effect::new(move |_| {
        if error.get().is_some() {
            reload.update(|n| *n += 1);
        }
    });

    let actions = CardActions {
        retry: Callback::new(move |()| reload.update(|n| *n += 1)),
        add: Callback::new(move |()| {
            notice.set(None);
            add_open.set(true);
        }),
        set_role: Callback::new(move |(principal, role): (PrincipalRef, Role)| {
            notice.set(None);
            error.set(None);
            let resource = resource.get_value();
            let done = Callback::new({
                let (label, role, id) =
                    (principal.label.clone(), role.clone(), resource.id.clone());
                move |()| {
                    notice.set(Some(format!("{label} {} {id}.", role.can_now())));
                    reload.update(|n| *n += 1);
                }
            });
            spawn_with_step_up(error, done, move || {
                set_resource_access_role(resource.clone(), principal.clone(), role.clone())
            });
        }),
        remove: Callback::new(move |principal: PrincipalRef| {
            notice.set(None);
            removing.set(Some(principal));
            remove_open.set(true);
        }),
    };

    let id = dialog_resource.id.clone();
    let on_added = Callback::new({
        let id = id.clone();
        move |(principal, role): (PrincipalRef, Role)| {
            notice.set(Some(format!(
                "{} {} {id}.",
                principal.label,
                role.can_now()
            )));
            reload.update(|n| *n += 1);
        }
    });
    let on_removed = Callback::new(move |principal: PrincipalRef| {
        notice.set(Some(format!(
            "{} no longer has access to {id}.",
            principal.label
        )));
        reload.update(|n| *n += 1);
    });
    let render = move |state: CardState| {
        resource.with_value(|r| card_body(r, state, stacked.get(), actions))
    };

    view! {
        <Card attr:data-testid="access-card">
            <Flex vertical=true gap=FlexGap::Medium padding=SpacingSize::Size200.inset()>
                {move || notice.get().map(|text| view! {
                    <span data-testid="access-card-notice">
                        <MessageBar intent=MessageBarIntent::Success>{text}</MessageBar>
                    </span>
                })}
                {move || error.get().map(|msg| {
                    let text = resource.with_value(|r| write_error_text(&msg, &r.noun));
                    view! {
                        <span data-testid="access-card-write-error">
                            <MessageBar intent=MessageBarIntent::Error>{text}</MessageBar>
                        </span>
                    }
                })}
                <Transition fallback=move || render(CardState::Loading)>
                    {move || load.get().map(|l| render(CardState::from_load(l)))}
                </Transition>
            </Flex>
        </Card>
        <AddAccessDialog
            open=add_open
            resource=dialog_resource.clone()
            roles=Signal::derive(move || latest.get().map(|v| v.roles).unwrap_or_default())
            existing=Signal::derive(move || latest.get().map(|v| v.entries).unwrap_or_default())
            on_added=on_added
        />
        <RemoveAccessDialog
            open=remove_open
            resource=dialog_resource
            principal=removing
            on_removed=on_removed
        />
    }
}
