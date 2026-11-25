//! [`RemoveAccessDialog`]: confirm taking a principal's access away.

use leptos::prelude::*;

use super::server::remove_resource_access;
use super::step_up::spawn_with_step_up;
use super::{is_last_owner, PrincipalRef, ResourceRef};
use crate::components::Body1;
use crate::primitives::{
    Button, ButtonAppearance, Dialog, DialogActions, DialogBody, DialogContent, DialogSurface,
    DialogTitle, Flex, FlexGap, MessageBar, MessageBarIntent,
};

/// The message for a failed write: the last-owner refusal in words, otherwise the server's.
#[must_use]
pub fn write_error_text(err: &str, noun: &str) -> String {
    if is_last_owner(&err) {
        format!("A {noun} needs at least one owner. Make someone else an owner first.")
    } else {
        err.to_string()
    }
}

/// The confirmation body: what removal does, and any error.
pub(crate) fn remove_access_body(
    name: &str,
    resource: &ResourceRef,
    error: Option<String>,
) -> impl IntoView {
    let message = format!(
        "{name} loses access to {} right away. Work already started keeps going.",
        resource.id
    );
    let error = error.map(|msg| {
        let text = write_error_text(&msg, &resource.noun);
        view! {
            <span data-testid="remove-access-error">
                <MessageBar intent=MessageBarIntent::Error>{text}</MessageBar>
            </span>
        }
    });
    view! {
        <Flex vertical=true gap=FlexGap::Medium attr:data-testid="remove-access-body">
            <Body1>{message}</Body1>
            {error}
        </Flex>
    }
}

/// Dialog that confirms removing `principal` from `resource`.
///
/// The backend refuses to remove the last owner; the dialog then says "A job needs at least
/// one owner. Make someone else an owner first." `on_removed` runs after the removal.
#[component]
pub fn RemoveAccessDialog(
    /// Whether the dialog is showing.
    open: RwSignal<bool>,
    /// The resource.
    resource: ResourceRef,
    /// Who is being removed.
    #[prop(into)]
    principal: Signal<Option<PrincipalRef>>,
    /// Runs with the removed principal.
    on_removed: Callback<PrincipalRef>,
) -> impl IntoView {
    let error = RwSignal::new(None::<String>);
    let resource = StoredValue::new(resource);

    Effect::new(move |_| {
        if open.get() {
            error.set(None);
        }
    });

    let confirm = move |_| {
        let Some(target) = principal.get() else {
            return;
        };
        let on_ok = Callback::new({
            let target = target.clone();
            move |()| {
                open.set(false);
                on_removed.run(target.clone());
            }
        });
        let resource = resource.get_value();
        spawn_with_step_up(error, on_ok, move || {
            remove_resource_access(resource.clone(), target.clone())
        });
    };
    let title = move || {
        principal
            .get()
            .map(|p| format!("Remove {}?", p.label))
            .unwrap_or_default()
    };

    view! {
        <Dialog open=open>
            <DialogSurface>
                <DialogBody>
                    <DialogTitle>{title}</DialogTitle>
                    <DialogContent>
                        {move || {
                            let name = principal.get().map(|p| p.label).unwrap_or_default();
                            resource.with_value(|r| remove_access_body(&name, r, error.get()))
                        }}
                    </DialogContent>
                    <DialogActions>
                        <Button
                            appearance=ButtonAppearance::Secondary
                            on_click=Callback::new(move |_| open.set(false))
                        >
                            "Cancel"
                        </Button>
                        <Button
                            appearance=ButtonAppearance::Primary
                            on_click=Callback::new(confirm)
                            attr:data-testid="remove-access-confirm"
                        >
                            "Remove"
                        </Button>
                    </DialogActions>
                </DialogBody>
            </DialogSurface>
        </Dialog>
    }
}
