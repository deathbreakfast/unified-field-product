//! [`AccessDenied`] and [`DeniedActionButton`]: what a reader sees where they lack an action.

use leptos::prelude::*;

use super::parts::{join_names, sent_caption, tint_badge};
use super::request_access_dialog::RequestAccessDialog;
use super::server::load_resource_access;
use super::{AccessLoad, AccessRequestReceipt, ResourceRef, Role};
use crate::components::{Body1, Caption2, EmptyState, EmptyStateCallToAction};
use crate::primitives::{Button, ButtonAppearance, Flex, FlexAlign, FlexGap, Tooltip};

/// What the denied views know about the resource's owners.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DeniedInfo {
    /// Names of the owners who review requests.
    pub owners: Vec<String>,
    /// The roles the kind offers.
    pub roles: Vec<Role>,
    /// True when the host wired no backend, so nobody can grant anything.
    pub unwired: bool,
}

impl DeniedInfo {
    /// Read owners and roles from a load. A failed load gives no names.
    #[must_use]
    pub fn from_load(load: &Result<AccessLoad, ServerFnError>) -> Self {
        match load {
            Ok(AccessLoad::Ready(view)) => Self {
                owners: view.owner_names(),
                roles: view.roles.clone(),
                unwired: false,
            },
            Ok(AccessLoad::Unwired) => Self {
                unwired: true,
                ..Self::default()
            },
            Err(_) => Self::default(),
        }
    }
}

/// "You need Run on billing-rollup to see this job's runs and logs."
#[must_use]
pub fn denied_sentence(action: &str, resource: &ResourceRef, purpose: Option<&str>) -> String {
    match purpose.map(str::trim).filter(|p| !p.is_empty()) {
        Some(purpose) => format!("You need {action} on {} to {purpose}.", resource.id),
        None => format!("You need {action} on {}.", resource.id),
    }
}

/// The tooltip on a button the reader can't use: "You need Run on billing-rollup. Owners: …"
#[must_use]
pub fn denied_tooltip(action: &str, resource: &ResourceRef, owners: &[String]) -> String {
    let need = denied_sentence(action, resource, None);
    if owners.is_empty() {
        need
    } else {
        format!("{need} Owners: {}.", join_names(owners))
    }
}

/// Loads owners and roles once per resource, for the denied views.
fn use_denied_info(
    resource: StoredValue<ResourceRef>,
) -> Resource<Result<AccessLoad, ServerFnError>> {
    Resource::new(move || resource.get_value(), load_resource_access)
}

/// The denied state's content for one set of facts. Pure, so each state can be rendered alone.
pub(crate) fn denied_view(
    resource: &ResourceRef,
    action: &str,
    purpose: Option<&str>,
    info: &DeniedInfo,
    sent: Option<AccessRequestReceipt>,
    on_request: Callback<()>,
) -> AnyView {
    let sentence = denied_sentence(action, resource, purpose);
    let reviewers = if info.unwired {
        Some("Access control isn't set up on this host, so nobody can grant this yet.".to_string())
    } else if info.owners.is_empty() {
        None
    } else {
        Some(format!(
            "Owners review requests: {}.",
            join_names(&info.owners)
        ))
    };
    let reviewers = reviewers.map(|text| {
        view! {
            <span data-testid="access-denied-reviewers">
                <Body1>{text}</Body1>
            </span>
        }
    });
    let action_area = match (sent, info.unwired) {
        (_, true) => None,
        (Some(receipt), false) => {
            let caption = sent_caption(receipt.sent_at, &info.owners);
            Some(
                view! {
                    <Flex gap=FlexGap::Small align=FlexAlign::Center attr:data-testid="access-denied-sent">
                        {tint_badge("Request sent", "access-denied-sent-badge")}
                        <Caption2>{caption}</Caption2>
                    </Flex>
                }
                .into_any(),
            )
        }
        (None, false) => Some(
            view! {
                <Button
                    appearance=ButtonAppearance::Primary
                    on_click=Callback::new(move |_| on_request.run(()))
                    attr:data-testid="access-denied-request"
                >
                    "Request access"
                </Button>
            }
            .into_any(),
        ),
    };
    view! {
        <div data-testid="access-denied" data-action=action.to_string()>
            <EmptyState message="Access needed" icon=icondata::AiLockOutlined>
                <EmptyStateCallToAction slot:call_to_action>
                    <Flex vertical=true gap=FlexGap::Small align=FlexAlign::Center>
                        <span data-testid="access-denied-message">
                            <Body1>{sentence}</Body1>
                        </span>
                        {reviewers}
                        {action_area}
                    </Flex>
                </EmptyStateCallToAction>
            </EmptyState>
        </div>
    }
    .into_any()
}

/// Stands in for the part of a page the reader can't use, never the whole page.
///
/// Names the missing action in words ("You need View on billing-rollup to see this job's runs
/// and logs.") and the owners who review requests, with Request access. After sending it
/// shows "Request sent" and the time, and offers no second request.
///
/// ```rust,ignore
/// use leptos::prelude::*;
/// use uf_product::access::{AccessDenied, ResourceRef};
///
/// let job = ResourceRef::new("chronon_job", "billing-rollup").noun("job");
/// view! { <AccessDenied resource=job action="View" purpose="see this job's runs and logs" /> };
/// ```
#[component]
pub fn AccessDenied(
    /// The resource.
    resource: ResourceRef,
    /// The missing action, such as `"View"` or `"Run"`.
    #[prop(into)]
    action: String,
    /// What the action would let the reader do here, such as "see this job's runs and logs".
    #[prop(optional, into)]
    purpose: Option<String>,
) -> impl IntoView {
    let dialog_open = RwSignal::new(false);
    let sent = RwSignal::new(None::<AccessRequestReceipt>);
    let role = Role::for_action(&action);
    let dialog_resource = resource.clone();
    let resource = StoredValue::new(resource);
    let action = StoredValue::new(action);
    let purpose = StoredValue::new(purpose);
    let load = use_denied_info(resource);
    let info = Signal::derive(move || {
        load.get()
            .map(|l| DeniedInfo::from_load(&l))
            .unwrap_or_default()
    });
    let on_request = Callback::new(move |()| dialog_open.set(true));
    let render = move |info: DeniedInfo| {
        resource.with_value(|r| {
            action.with_value(|a| {
                purpose
                    .with_value(|p| denied_view(r, a, p.as_deref(), &info, sent.get(), on_request))
            })
        })
    };

    view! {
        <Suspense fallback=move || render(DeniedInfo::default())>
            {move || load.get().map(|l| render(DeniedInfo::from_load(&l)))}
        </Suspense>
        <RequestAccessDialog
            open=dialog_open
            resource=dialog_resource
            role=role
            roles=Signal::derive(move || info.get().roles)
            owners=Signal::derive(move || info.get().owners)
            on_sent=Callback::new(move |receipt| sent.set(Some(receipt)))
        />
    }
}

/// The disabled button, its tooltip, and the "Ask for … access" link (or "Request sent").
pub(crate) fn denied_action_view(
    label: &str,
    action: &str,
    tooltip: Signal<String>,
    sent: bool,
    on_ask: Callback<()>,
) -> AnyView {
    let label = label.to_string();
    let ask = if sent {
        view! {
            <span data-testid="denied-action-sent">
                <Caption2>"Request sent"</Caption2>
            </span>
        }
        .into_any()
    } else {
        let text = format!("Ask for {action} access");
        view! {
            <Button
                appearance=ButtonAppearance::Transparent
                on_click=Callback::new(move |_| on_ask.run(()))
                attr:data-testid="denied-action-ask"
            >
                {text}
            </Button>
        }
        .into_any()
    };
    view! {
        <Flex gap=FlexGap::Small align=FlexAlign::Center attr:data-testid="denied-action">
            <Tooltip content=tooltip>
                <Button
                    appearance=ButtonAppearance::Primary
                    disabled=true
                    disabled_focusable=true
                    attr:data-testid="denied-action-button"
                >
                    {label}
                </Button>
            </Tooltip>
            {ask}
        </Flex>
    }
    .into_any()
}

/// A button for an action the reader lacks: visible and disabled, with a tooltip naming the
/// action and the owners, and an "Ask for … access" link beside it that opens the request.
///
/// ```rust,ignore
/// use leptos::prelude::*;
/// use uf_product::access::{DeniedActionButton, ResourceRef};
///
/// let job = ResourceRef::new("chronon_job", "billing-rollup").noun("job");
/// view! { <DeniedActionButton resource=job action="Run" label="Run now" /> };
/// ```
#[component]
pub fn DeniedActionButton(
    /// The resource.
    resource: ResourceRef,
    /// The missing action, such as `"Run"`.
    #[prop(into)]
    action: String,
    /// The button's text, such as "Run now".
    #[prop(into)]
    label: String,
) -> impl IntoView {
    let dialog_open = RwSignal::new(false);
    let sent = RwSignal::new(false);
    let role = Role::for_action(&action);
    let dialog_resource = resource.clone();
    let resource = StoredValue::new(resource);
    let load = use_denied_info(resource);
    let info = Signal::derive(move || {
        load.get()
            .map(|l| DeniedInfo::from_load(&l))
            .unwrap_or_default()
    });
    let tooltip = {
        let action = action.clone();
        Signal::derive(move || {
            resource.with_value(|r| denied_tooltip(&action, r, &info.get().owners))
        })
    };
    let action = StoredValue::new(action);
    let label = StoredValue::new(label);
    let on_ask = Callback::new(move |()| dialog_open.set(true));

    view! {
        {move || action.with_value(|a| {
            label.with_value(|l| denied_action_view(l, a, tooltip, sent.get(), on_ask))
        })}
        <RequestAccessDialog
            open=dialog_open
            resource=dialog_resource
            role=role
            roles=Signal::derive(move || info.get().roles)
            owners=Signal::derive(move || info.get().owners)
            on_sent=Callback::new(move |_| sent.set(true))
        />
    }
}
