//! The resource-access contract: resources, roles, principals, and the host-wired backend.

#[cfg(feature = "ssr")]
use std::sync::Arc;

#[cfg(feature = "ssr")]
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// One resource whose access the shared views show, such as a Chronon job.
///
/// `kind` is the resource kind's stable prefix (`"chronon_job"`), `id` the resource id within
/// that kind, `label` the display name, and `noun` the word the views use for the kind in
/// sentences ("Super User can always manage every job").
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceRef {
    /// Kind prefix, such as `"chronon_job"`.
    pub kind: String,
    /// Resource id within the kind, such as `"billing-rollup"`.
    pub id: String,
    /// Display name, such as `"Billing rollup"`. Defaults to `id`.
    pub label: String,
    /// The kind in a sentence, such as `"job"`. Defaults to `"resource"`.
    pub noun: String,
}

impl ResourceRef {
    /// A resource of `kind` with `id`, labelled by its id and called a "resource".
    #[must_use]
    pub fn new(kind: impl Into<String>, id: impl Into<String>) -> Self {
        let id = id.into();
        Self {
            kind: kind.into(),
            label: id.clone(),
            id,
            noun: "resource".to_string(),
        }
    }

    /// Set the display name.
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Set the word used for the kind in sentences ("job", "table").
    #[must_use]
    pub fn noun(mut self, noun: impl Into<String>) -> Self {
        self.noun = noun.into();
        self
    }

    /// The permission name for `action` on this resource: `{kind}.{id}.{action}`.
    #[must_use]
    pub fn permission_name(&self, action: &str) -> String {
        format!("{}.{}.{}", self.kind, self.id, action)
    }

    /// `"Billing rollup (billing-rollup)"`, or just the id when the label is the id.
    #[must_use]
    pub fn label_with_id(&self) -> String {
        if self.label.is_empty() || self.label == self.id {
            self.id.clone()
        } else {
            format!("{} ({})", self.label, self.id)
        }
    }
}

/// What a principal may do on one resource, as a fixed set of grants.
///
/// The kind decides which roles it offers (a Spectra table has Read unmasked, a Photon topic
/// has no Edit). `Custom` is a set of grants that matches no role, listed by action name.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    /// View.
    View,
    /// View and Run.
    Run,
    /// View and Unmask.
    ReadUnmasked,
    /// View, Run (when the kind has it) and Edit.
    Edit,
    /// Every action, including changing who has access.
    Owner,
    /// Grants that match no role, by action name.
    Custom(Vec<String>),
}

impl Role {
    /// The roles most kinds offer, in Select order.
    pub const STANDARD: [Role; 4] = [Role::View, Role::Run, Role::Edit, Role::Owner];

    /// The role's name in a Select: "Can view", "Can run", "Owner".
    #[must_use]
    pub fn label(&self) -> String {
        match self {
            Self::View => "Can view".to_string(),
            Self::Run => "Can run".to_string(),
            Self::ReadUnmasked => "Can read unmasked".to_string(),
            Self::Edit => "Can edit".to_string(),
            Self::Owner => "Owner".to_string(),
            Self::Custom(actions) if actions.is_empty() => "Custom".to_string(),
            Self::Custom(actions) => format!("Custom: {}", actions.join(", ")),
        }
    }

    /// Stable value for a Select option. Every `Custom` role shares `"custom"`.
    #[must_use]
    pub const fn key(&self) -> &'static str {
        match self {
            Self::View => "view",
            Self::Run => "run",
            Self::ReadUnmasked => "read_unmasked",
            Self::Edit => "edit",
            Self::Owner => "owner",
            Self::Custom(_) => "custom",
        }
    }

    /// The role a [`Role::key`] names. `"custom"` and unknown keys give `None`.
    #[must_use]
    pub fn from_key(key: &str) -> Option<Self> {
        match key {
            "view" => Some(Self::View),
            "run" => Some(Self::Run),
            "read_unmasked" => Some(Self::ReadUnmasked),
            "edit" => Some(Self::Edit),
            "owner" => Some(Self::Owner),
            _ => None,
        }
    }

    /// The smallest role that holds `action` ("View", "Run", "Unmask", "Edit"; anything else
    /// needs Owner).
    #[must_use]
    pub fn for_action(action: &str) -> Self {
        match action {
            "View" => Self::View,
            "Run" => Self::Run,
            "Unmask" => Self::ReadUnmasked,
            "Edit" => Self::Edit,
            _ => Self::Owner,
        }
    }

    /// What the role allows, for the line under the role Select.
    #[must_use]
    pub fn describe(&self, noun: &str) -> String {
        match self {
            Self::View => format!("Can view: see the {noun} and its history."),
            Self::Run => {
                format!("Can run: see the {noun} and its runs, and start, retry or cancel a run.")
            }
            Self::ReadUnmasked => {
                format!("Can read unmasked: see the {noun} with personal fields shown.")
            }
            Self::Edit => format!("Can edit: run the {noun} and change its settings."),
            Self::Owner => {
                format!("Owner: everything on the {noun}, including who has access.")
            }
            Self::Custom(actions) => format!(
                "Custom: holds {} on the {noun}, set outside these roles.",
                actions.join(", ")
            ),
        }
    }

    /// The reader's own role as a sentence: "You can edit this job."
    #[must_use]
    pub fn you_can(&self, noun: &str) -> String {
        match self {
            Self::View => format!("You can view this {noun}."),
            Self::Run => format!("You can run this {noun}."),
            Self::ReadUnmasked => format!("You can read this {noun} unmasked."),
            Self::Edit => format!("You can edit this {noun}."),
            Self::Owner => format!("You own this {noun}."),
            Self::Custom(actions) => {
                format!("You hold {} on this {noun}.", actions.join(", "))
            }
        }
    }

    /// What a principal given this role can now do: "can now run".
    #[must_use]
    pub fn can_now(&self) -> String {
        match self {
            Self::View => "can now view".to_string(),
            Self::Run => "can now run".to_string(),
            Self::ReadUnmasked => "can now read unmasked".to_string(),
            Self::Edit => "can now edit".to_string(),
            Self::Owner => "now owns".to_string(),
            Self::Custom(actions) => format!("now holds {} on", actions.join(", ")),
        }
    }
}

/// What kind of principal a row names.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrincipalKind {
    /// A person.
    User,
    /// A permission group.
    Group,
    /// A group another product manages from a team roster.
    Team {
        /// The team's organization, when the host knows it.
        org: Option<String>,
        /// How many people are on the team.
        member_count: u32,
    },
    /// A service user.
    ServiceUser {
        /// Who owns the service user, when the host knows it.
        owner: Option<String>,
    },
}

impl PrincipalKind {
    /// Stable short name: `user`, `group`, `team`, `service_user`.
    #[must_use]
    pub const fn key(&self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Group => "group",
            Self::Team { .. } => "team",
            Self::ServiceUser { .. } => "service_user",
        }
    }
}

/// A person, group, team group or service user that can hold a role.
///
/// `label` is a display name (people as "Name (id6)"), never an email.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PrincipalRef {
    /// What kind of principal this is.
    pub kind: PrincipalKind,
    /// Stable id within its kind.
    pub id: String,
    /// Display name.
    pub label: String,
}

impl PrincipalRef {
    /// Unique key across kinds: `{kind}:{id}`.
    #[must_use]
    pub fn key(&self) -> String {
        format!("{}:{}", self.kind.key(), self.id)
    }

    /// The line under the name: team size, service-user owner, or nothing.
    #[must_use]
    pub fn secondary(&self) -> Option<String> {
        match &self.kind {
            PrincipalKind::Team { org, member_count } => {
                let size = if *member_count == 1 {
                    "1 member".to_string()
                } else {
                    format!("{member_count} members")
                };
                Some(match org {
                    Some(org) => format!("{org} · {size}"),
                    None => size,
                })
            }
            PrincipalKind::ServiceUser { owner: Some(owner) } => {
                Some(format!("Service user · owned by {owner}"))
            }
            PrincipalKind::ServiceUser { owner: None } => Some("Service user".to_string()),
            PrincipalKind::User | PrincipalKind::Group => None,
        }
    }
}

/// One row of the Access card.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessEntry {
    /// Who holds the role.
    pub principal: PrincipalRef,
    /// The role they hold.
    pub role: Role,
    /// The app's admin group on a code-declared resource.
    pub default_owner: bool,
    /// A kind-wide group; removing it restricts the resource to the other rows.
    pub umbrella: bool,
}

impl AccessEntry {
    /// True when this row makes its principal one of the resource's owners for "yours first"
    /// lists: Owner or Can edit (or a custom set that includes Edit), and not the app admin
    /// group's default ownership.
    #[must_use]
    pub fn counts_as_owned(&self) -> bool {
        if self.default_owner {
            return false;
        }
        match &self.role {
            Role::Owner | Role::Edit => true,
            Role::Custom(actions) => actions.iter().any(|a| a == "Edit"),
            Role::View | Role::Run | Role::ReadUnmasked => false,
        }
    }
}

/// Ids of the resources where one of `principals` holds a row that
/// [counts as owned](AccessEntry::counts_as_owned).
///
/// `principals` is the actor plus every group and managed team group they're in. Backends
/// implementing [`ResourceAccessBackend::owned_by`] read each resource's rows and pass them
/// here, so every host applies the same rule. Ids come back in input order.
#[must_use]
pub fn owned_resource_ids<'a, I, E>(resources: I, principals: &[PrincipalRef]) -> Vec<String>
where
    I: IntoIterator<Item = (&'a str, E)>,
    E: IntoIterator<Item = &'a AccessEntry>,
{
    let keys: std::collections::HashSet<String> =
        principals.iter().map(PrincipalRef::key).collect();
    resources
        .into_iter()
        .filter_map(|(id, entries)| {
            entries
                .into_iter()
                .any(|e| e.counts_as_owned() && keys.contains(&e.principal.key()))
                .then(|| id.to_string())
        })
        .collect()
}

/// Who can act on one resource, as the reader sees it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessView {
    /// Every row, in display order.
    pub entries: Vec<AccessEntry>,
    /// The roles this kind offers, in Select order.
    pub roles: Vec<Role>,
    /// What the reader holds, if anything.
    pub viewer_role: Option<Role>,
    /// Whether the reader may change who has access.
    pub can_maintain: bool,
}

impl AccessView {
    /// Names of the rows that own the resource, who review access requests.
    #[must_use]
    pub fn owner_names(&self) -> Vec<String> {
        self.entries
            .iter()
            .filter(|e| e.role == Role::Owner)
            .map(|e| e.principal.label.clone())
            .collect()
    }

    /// True when no kind-wide row is left, so only the listed rows can see the resource.
    #[must_use]
    pub fn restricted(&self) -> bool {
        !self.entries.iter().any(|e| e.umbrella)
    }

    /// The role `principal` holds here, if they have a row.
    #[must_use]
    pub fn role_of(&self, principal: &PrincipalRef) -> Option<&Role> {
        let key = principal.key();
        self.entries
            .iter()
            .find(|e| e.principal.key() == key)
            .map(|e| &e.role)
    }
}

/// Search results for the principal picker, one list per group.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrincipalResults {
    /// Team groups managed from a team roster. Empty on hosts with no managed teams.
    pub teams: Vec<PrincipalRef>,
    /// Permission groups.
    pub groups: Vec<PrincipalRef>,
    /// People and service users.
    pub people: Vec<PrincipalRef>,
}

/// Id of an access request the backend created.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RequestId(pub String);

/// A sent access request and when it was sent (unix seconds, UTC).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccessRequestReceipt {
    /// The request's id.
    pub id: RequestId,
    /// When it was sent, in unix seconds.
    pub sent_at: i64,
}

/// What loading a resource's access returned.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessLoad {
    /// The backend answered.
    Ready(AccessView),
    /// No backend is wired on this host.
    Unwired,
}

/// Why a resource-access call failed.
///
/// The `Display` text is what reaches the browser. Stable prefixes let the views tell the
/// cases apart: `ACCESS:denied:`, `ACCESS:last_owner:`, `ACCESS:unwired:`, and the step-up
/// message unchanged (`STEP_UP:…`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessError {
    /// The actor lacks `action` on the resource.
    Denied {
        /// The missing action, such as `"Run"`.
        action: String,
        /// The resource id or label.
        resource: String,
    },
    /// The change would leave the resource with no owner.
    LastOwner {
        /// The resource id or label.
        resource: String,
    },
    /// The change needs a recent authenticator code; carries the `STEP_UP:` message.
    StepUp(String),
    /// No backend is wired on this host.
    Unwired,
    /// The backend or its store failed.
    Backend {
        /// What was being done, such as `"load access"`.
        op: String,
        /// The underlying error.
        message: String,
    },
}

/// Prefix of [`AccessError::LastOwner`] messages.
pub const LAST_OWNER_PREFIX: &str = "ACCESS:last_owner:";
/// Prefix of [`AccessError::Unwired`] messages.
pub const UNWIRED_PREFIX: &str = "ACCESS:unwired:";
/// Prefix of [`AccessError::Denied`] messages.
pub const DENIED_PREFIX: &str = "ACCESS:denied:";

impl std::fmt::Display for AccessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Denied { action, resource } => {
                write!(f, "{DENIED_PREFIX} You need {action} on {resource}.")
            }
            Self::LastOwner { resource } => {
                write!(
                    f,
                    "{LAST_OWNER_PREFIX} {resource} needs at least one owner."
                )
            }
            Self::StepUp(message) => f.write_str(message),
            Self::Unwired => {
                write!(
                    f,
                    "{UNWIRED_PREFIX} Access control isn't set up on this host."
                )
            }
            Self::Backend { op, message } => write!(f, "Failed to {op}: {message}"),
        }
    }
}

impl std::error::Error for AccessError {}

/// True when `err` (a server error's text) is the last-owner refusal.
#[must_use]
pub fn is_last_owner(err: &impl std::fmt::Display) -> bool {
    err.to_string().contains(LAST_OWNER_PREFIX)
}

/// Evaluates and changes who can act on one resource, for the current request actor.
///
/// Every method runs as the session actor. Implementations enforce their own checks: `access`
/// returns the rows the reader may see (at least the owner rows, so a reader without View
/// can still be told who reviews requests), `set_role` and `remove` need the resource's
/// Maintain action and refuse to leave it with no owner ([`AccessError::LastOwner`]).
///
/// A host that wires nothing gets a fail-closed backend: [`require`] refuses and the views
/// say "Access control isn't set up on this host".
#[cfg(feature = "ssr")]
#[async_trait::async_trait]
pub trait ResourceAccessBackend: Send + Sync {
    /// Who can act on `resource`, as the reader sees it.
    async fn access(&self, resource: &ResourceRef) -> Result<AccessView, AccessError>;
    /// Give `principal` exactly `role` on `resource`, replacing their grants in one write.
    async fn set_role(
        &self,
        resource: &ResourceRef,
        principal: &PrincipalRef,
        role: Role,
    ) -> Result<(), AccessError>;
    /// Take every grant on `resource` away from `principal`.
    async fn remove(
        &self,
        resource: &ResourceRef,
        principal: &PrincipalRef,
    ) -> Result<(), AccessError>;
    /// Principals whose name matches `query`, grouped for the picker.
    async fn search_principals(&self, query: &str) -> Result<PrincipalResults, AccessError>;
    /// Ask the resource's owners for `role`, with the reader's `reason`.
    async fn request(
        &self,
        resource: &ResourceRef,
        role: Role,
        reason: &str,
    ) -> Result<RequestId, AccessError>;
    /// Whether the actor holds `action` (such as `"Run"`) on `resource`.
    async fn can(&self, resource: &ResourceRef, action: &str) -> Result<bool, AccessError>;
    /// Ids of the resources of `kind` that `actor`, or a group or managed team group they're
    /// in, holds Owner or Can edit on, for "yours first" lists.
    ///
    /// Can run and Can view don't count, and neither does the app admin group's default
    /// ownership ([`AccessEntry::default_owner`]), so an admin still sees their own team's
    /// work first. [`owned_resource_ids`] applies that rule to rows the backend has read. The
    /// default body returns an empty list, which leaves lists in their usual order.
    async fn owned_by(&self, actor: &PrincipalRef, kind: &str) -> Result<Vec<String>, AccessError> {
        let _ = (actor, kind);
        Ok(Vec::new())
    }
}

/// Install the resource-access backend for this Leptos runtime (host bootstrap).
#[cfg(feature = "ssr")]
pub fn provide_resource_access_backend(backend: Arc<dyn ResourceAccessBackend>) {
    provide_context(backend);
}

/// Read the installed backend, if any.
#[cfg(feature = "ssr")]
pub fn use_resource_access_backend() -> Option<Arc<dyn ResourceAccessBackend>> {
    use_context::<Arc<dyn ResourceAccessBackend>>()
}

/// The installed backend, or the fail-closed one when the host wired none.
#[cfg(feature = "ssr")]
pub(crate) fn backend_or_unwired() -> Arc<dyn ResourceAccessBackend> {
    use_resource_access_backend().unwrap_or_else(|| Arc::new(UnwiredBackend))
}

/// Stands in when no backend is wired: nobody can do anything, and every read says why.
#[cfg(feature = "ssr")]
pub(crate) struct UnwiredBackend;

#[cfg(feature = "ssr")]
#[async_trait::async_trait]
impl ResourceAccessBackend for UnwiredBackend {
    async fn access(&self, _resource: &ResourceRef) -> Result<AccessView, AccessError> {
        Err(AccessError::Unwired)
    }

    async fn set_role(
        &self,
        _resource: &ResourceRef,
        _principal: &PrincipalRef,
        _role: Role,
    ) -> Result<(), AccessError> {
        Err(AccessError::Unwired)
    }

    async fn remove(
        &self,
        _resource: &ResourceRef,
        _principal: &PrincipalRef,
    ) -> Result<(), AccessError> {
        Err(AccessError::Unwired)
    }

    async fn search_principals(&self, _query: &str) -> Result<PrincipalResults, AccessError> {
        Err(AccessError::Unwired)
    }

    async fn request(
        &self,
        _resource: &ResourceRef,
        _role: Role,
        _reason: &str,
    ) -> Result<RequestId, AccessError> {
        Err(AccessError::Unwired)
    }

    async fn can(&self, _resource: &ResourceRef, _action: &str) -> Result<bool, AccessError> {
        Ok(false)
    }

    async fn owned_by(
        &self,
        _actor: &PrincipalRef,
        _kind: &str,
    ) -> Result<Vec<String>, AccessError> {
        Ok(Vec::new())
    }
}

/// Fail-closed gate for server functions: the session actor must hold `action` on `resource`.
///
/// # Errors
///
/// Returns [`ServerFnError`] with Higgs' permission-denied payload (naming
/// `{kind}.{id}.{action}`) when the actor lacks the action or no backend is wired, and the
/// check-failed payload when the backend errors.
#[cfg(feature = "ssr")]
pub async fn require(resource: &ResourceRef, action: &str) -> Result<(), ServerFnError> {
    require_with(backend_or_unwired().as_ref(), resource, action).await
}

#[cfg(feature = "ssr")]
pub(crate) async fn require_with(
    backend: &dyn ResourceAccessBackend,
    resource: &ResourceRef,
    action: &str,
) -> Result<(), ServerFnError> {
    let name = resource.permission_name(action);
    match backend.can(resource, action).await {
        Ok(true) => Ok(()),
        Ok(false) => Err(ServerFnError::new(
            higgs::server_runtime::permission_denied_payload(&name),
        )),
        Err(e) => Err(ServerFnError::new(
            higgs::server_runtime::permission_check_failed_payload(&name, &e.to_string()),
        )),
    }
}
