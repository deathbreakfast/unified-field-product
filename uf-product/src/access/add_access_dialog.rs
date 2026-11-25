//! [`AddAccessDialog`]: give a team, group or person a role on one resource.

use leptos::prelude::*;

use super::principal_picker::PrincipalPicker;
use super::server::set_resource_access_role;
use super::step_up::spawn_with_step_up;
use super::{AccessEntry, PrincipalRef, ResourceRef, Role};
use crate::components::{Body1, SelectOption};
use crate::primitives::{
    Button, ButtonAppearance, Dialog, DialogActions, DialogBody, DialogContent, DialogSurface,
    DialogTitle, Field, Flex, FlexGap, MessageBar, MessageBarIntent, Select,
};

/// The role the dialog starts on: Can run when the kind offers it, otherwise Can view.
#[must_use]
pub fn default_role(roles: &[Role]) -> Role {
    if roles.contains(&Role::Run) {
        Role::Run
    } else {
        Role::View
    }
}

/// Role options for a Select: the kind's roles without Custom, or the standard four.
pub(crate) fn role_options(roles: &[Role]) -> Vec<Role> {
    let offered: Vec<Role> = roles
        .iter()
        .filter(|r| !matches!(r, Role::Custom(_)))
        .cloned()
        .collect();
    if offered.is_empty() {
        Role::STANDARD.to_vec()
    } else {
        offered
    }
}

/// The dialog's fields: who, which role and what it allows, and any save error.
pub(crate) fn add_access_fields(
    noun: String,
    who: RwSignal<Option<PrincipalRef>>,
    role_key: RwSignal<String>,
    roles: Vec<Role>,
    existing: Signal<Vec<AccessEntry>>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let options = role_options(&roles)
        .into_iter()
        .map(|role| view! { <SelectOption value=role.key() label=role.label() /> })
        .collect_view();
    let describe = move || {
        Role::from_key(&role_key.get())
            .map(|role| role.describe(&noun))
            .unwrap_or_default()
    };
    view! {
        <Flex vertical=true gap=FlexGap::Medium attr:data-testid="add-access-fields">
            <Field label="Who">
                <PrincipalPicker value=who existing=existing />
            </Field>
            <Field label="Role">
                <Select bind=role_key>{options}</Select>
            </Field>
            <span data-testid="add-access-role-help">
                <Body1>{describe}</Body1>
            </span>
            {move || error.get().map(|msg| view! {
                <span data-testid="add-access-error">
                    <MessageBar intent=MessageBarIntent::Error>{msg}</MessageBar>
                </span>
            })}
        </Flex>
    }
}

/// Dialog that gives one principal a role on `resource`.
///
/// The role starts on Can run when the kind offers it. Saving asks for a recent authenticator
/// code when the server wants one. A save error stays in the dialog with the choice kept.
/// `on_added` runs with the principal and role after the save.
#[component]
pub fn AddAccessDialog(
    /// Whether the dialog is showing.
    open: RwSignal<bool>,
    /// The resource being shared.
    resource: ResourceRef,
    /// The roles the kind offers.
    #[prop(into)]
    roles: Signal<Vec<Role>>,
    /// Rows already on the card.
    #[prop(into)]
    existing: Signal<Vec<AccessEntry>>,
    /// Runs after a successful save.
    on_added: Callback<(PrincipalRef, Role)>,
) -> impl IntoView {
    let who = RwSignal::new(None::<PrincipalRef>);
    let role_key = RwSignal::new(default_role(&roles.get_untracked()).key().to_string());
    let error = RwSignal::new(None::<String>);
    let saving = RwSignal::new(false);
    let title = format!("Give access to {}", resource.id);
    let noun = resource.noun.clone();
    let resource = StoredValue::new(resource);

    // Start each opening fresh, on the kind's default role.
    Effect::new(move |_| {
        if open.get() {
            who.set(None);
            error.set(None);
            role_key.set(default_role(&roles.get_untracked()).key().to_string());
        }
    });

    let close = move || {
        open.set(false);
        error.set(None);
    };
    let save = move |_| {
        let Some(principal) = who.get() else {
            error.set(Some("Pick a team, group or person first.".to_string()));
            return;
        };
        let Some(role) = Role::from_key(&role_key.get()) else {
            return;
        };
        saving.set(true);
        let on_ok = Callback::new({
            let (principal, role) = (principal.clone(), role.clone());
            move |()| {
                saving.set(false);
                open.set(false);
                on_added.run((principal.clone(), role.clone()));
            }
        });
        let resource = resource.get_value();
        spawn_with_step_up(error, on_ok, move || {
            set_resource_access_role(resource.clone(), principal.clone(), role.clone())
        });
    };
    // A failed save leaves the button usable again.
    Effect::new(move |_| {
        if error.get().is_some() {
            saving.set(false);
        }
    });

    view! {
        <Dialog open=open>
            <DialogSurface>
                <DialogBody>
                    <DialogTitle>{title}</DialogTitle>
                    <DialogContent>
                        {move || add_access_fields(
                            noun.clone(),
                            who,
                            role_key,
                            roles.get(),
                            existing,
                            error,
                        )}
                    </DialogContent>
                    <DialogActions>
                        <Button
                            appearance=ButtonAppearance::Secondary
                            on_click=Callback::new(move |_| close())
                        >
                            "Cancel"
                        </Button>
                        <Button
                            appearance=ButtonAppearance::Primary
                            disabled=Signal::derive(move || who.get().is_none() || saving.get())
                            loading=saving
                            on_click=Callback::new(save)
                            attr:data-testid="add-access-save"
                        >
                            "Add"
                        </Button>
                    </DialogActions>
                </DialogBody>
            </DialogSurface>
        </Dialog>
    }
}
