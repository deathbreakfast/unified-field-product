//! [`RequestAccessDialog`]: ask a resource's owners for a role.

use leptos::prelude::*;
use leptos::task::spawn_local_scoped;

use super::add_access_dialog::role_options;
use super::parts::join_names;
use super::server::request_resource_access;
use super::{AccessRequestReceipt, ResourceRef, Role};
use crate::components::{Body1, SelectOption};
use crate::primitives::{
    Button, ButtonAppearance, Dialog, DialogActions, DialogBody, DialogContent, DialogSurface,
    DialogTitle, Field, Flex, FlexGap, Input, InputAppearance, MessageBar, MessageBarIntent,
    Select, Textarea,
};

/// "job" → "Job".
fn capitalized(noun: &str) -> String {
    let mut chars = noun.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    })
}

/// Who a request goes to: "Goes to the owners of billing-rollup: Chronon admins. …"
#[must_use]
pub fn goes_to_caption(resource: &ResourceRef, owners: &[String]) -> String {
    if owners.is_empty() {
        format!(
            "Goes to the owners of {}. You will get a notification when they decide.",
            resource.id
        )
    } else {
        format!(
            "Goes to the owners of {}: {}. You will get a notification when they decide.",
            resource.id,
            join_names(owners)
        )
    }
}

/// The dialog's fields: the resource and role filled in, a reason, who it goes to, any error.
pub(crate) fn request_access_fields(
    resource: &ResourceRef,
    role_key: RwSignal<String>,
    roles: &[Role],
    reason: RwSignal<String>,
    owners: &[String],
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let resource_value = RwSignal::new(resource.label_with_id());
    let options = role_options(roles)
        .into_iter()
        .map(|role| view! { <SelectOption value=role.key() label=role.label() /> })
        .collect_view();
    let caption = goes_to_caption(resource, owners);
    let field_label = capitalized(&resource.noun);
    view! {
        <Flex vertical=true gap=FlexGap::Medium attr:data-testid="request-access-fields">
            <Field label=field_label>
                <Input
                    bind=resource_value
                    appearance=InputAppearance {
                        readonly: Signal::from(true),
                        ..Default::default()
                    }
                />
            </Field>
            <Field label="Access">
                <Select bind=role_key>{options}</Select>
            </Field>
            <Field label="Why do you need it?">
                <Textarea bind=reason />
            </Field>
            <span data-testid="request-access-goes-to">
                <Body1>{caption}</Body1>
            </span>
            {move || error.get().map(|msg| view! {
                <span data-testid="request-access-error">
                    <MessageBar intent=MessageBarIntent::Error>{msg}</MessageBar>
                </span>
            })}
        </Flex>
    }
}

/// Dialog that asks the owners of `resource` for a role.
///
/// The resource and role come filled in; the reader adds a reason. An error stays in the
/// dialog with the reason kept. `on_sent` runs with the receipt after sending.
#[component]
pub fn RequestAccessDialog(
    /// Whether the dialog is showing.
    open: RwSignal<bool>,
    /// The resource.
    resource: ResourceRef,
    /// The role to fill in.
    #[prop(into)]
    role: Signal<Role>,
    /// The roles the kind offers; the standard four when empty.
    #[prop(optional, into)]
    roles: Signal<Vec<Role>>,
    /// Names of the owners who review the request.
    #[prop(optional, into)]
    owners: Signal<Vec<String>>,
    /// Runs after the request is sent.
    on_sent: Callback<AccessRequestReceipt>,
) -> impl IntoView {
    let role_key = RwSignal::new(role.get_untracked().key().to_string());
    let reason = RwSignal::new(String::new());
    let error = RwSignal::new(None::<String>);
    let sending = RwSignal::new(false);
    let resource = StoredValue::new(resource);

    Effect::new(move |_| {
        if open.get() {
            role_key.set(role.get_untracked().key().to_string());
            error.set(None);
        }
    });

    let send = move |_| {
        let text = reason.get();
        if text.trim().is_empty() {
            error.set(Some("Add a reason so the owners can decide.".to_string()));
            return;
        }
        let Some(role) = Role::from_key(&role_key.get()) else {
            return;
        };
        sending.set(true);
        let resource = resource.get_value();
        spawn_local_scoped(async move {
            match request_resource_access(resource, role, text).await {
                Ok(receipt) => {
                    open.set(false);
                    error.set(None);
                    reason.set(String::new());
                    on_sent.run(receipt);
                }
                Err(e) => error.set(Some(e.to_string())),
            }
            sending.set(false);
        });
    };

    view! {
        <Dialog open=open>
            <DialogSurface>
                <DialogBody>
                    <DialogTitle>"Request access"</DialogTitle>
                    <DialogContent>
                        {move || {
                            let (roles, owners) = (roles.get(), owners.get());
                            resource.with_value(|r| {
                                request_access_fields(r, role_key, &roles, reason, &owners, error)
                            })
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
                            loading=sending
                            disabled=sending
                            on_click=Callback::new(send)
                            attr:data-testid="request-access-send"
                        >
                            "Send request"
                        </Button>
                    </DialogActions>
                </DialogBody>
            </DialogSurface>
        </Dialog>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capitalized_uppercases_first_letter() {
        assert_eq!(capitalized("job"), "Job");
        assert_eq!(capitalized(""), "");
    }
}
