//! Server functions the access views call. Each one runs as the session actor through the
//! installed [`super::ResourceAccessBackend`], or the fail-closed one when none is wired.

use leptos::prelude::*;
use leptos::server_fn::codec::Json;

use super::{AccessLoad, AccessRequestReceipt, PrincipalRef, PrincipalResults, ResourceRef, Role};

/// Queries shorter than this return no results, so the directory is never listed whole.
pub const MIN_QUERY_CHARS: usize = 2;
/// Longest reason a request may carry.
pub const MAX_REASON_CHARS: usize = 2000;

/// Who can act on `resource`, or [`AccessLoad::Unwired`] when the host wired no backend.
///
/// # Errors
///
/// Returns the backend's error text (such as `Failed to load access: …`).
#[server(LoadResourceAccess, input = Json)]
pub async fn load_resource_access(resource: ResourceRef) -> Result<AccessLoad, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        match super::backend::backend_or_unwired().access(&resource).await {
            Ok(view) => Ok(AccessLoad::Ready(view)),
            Err(super::AccessError::Unwired) => Ok(AccessLoad::Unwired),
            Err(e) => Err(ServerFnError::new(e.to_string())),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = resource;
        Err(ServerFnError::new(
            "load_resource_access requires the ssr feature",
        ))
    }
}

/// Give `principal` exactly `role` on `resource`. Used for Add people and role changes.
///
/// A role change is a grant, so it needs a recent authenticator code first.
///
/// # Errors
///
/// Returns a `STEP_UP:` error when the code window is missing or expired, a `Custom` role
/// refusal, or the backend's error (`ACCESS:last_owner:` when it would leave no owner).
#[server(SetResourceAccessRole, input = Json)]
pub async fn set_resource_access_role(
    resource: ResourceRef,
    principal: PrincipalRef,
    role: Role,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        if matches!(role, Role::Custom(_)) {
            return Err(ServerFnError::new(
                "Custom grants can't be set from here. Pick one of the roles.",
            ));
        }
        crate::permissions::require_step_up("window").await?;
        super::backend::backend_or_unwired()
            .set_role(&resource, &principal, role)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = (resource, principal, role);
        Err(ServerFnError::new(
            "set_resource_access_role requires the ssr feature",
        ))
    }
}

/// Take every grant on `resource` away from `principal`.
///
/// # Errors
///
/// Returns the backend's error (`ACCESS:last_owner:` when it would leave no owner).
#[server(RemoveResourceAccess, input = Json)]
pub async fn remove_resource_access(
    resource: ResourceRef,
    principal: PrincipalRef,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        super::backend::backend_or_unwired()
            .remove(&resource, &principal)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = (resource, principal);
        Err(ServerFnError::new(
            "remove_resource_access requires the ssr feature",
        ))
    }
}

/// Principals matching `query`, grouped Teams, Groups, People. Short queries return nothing.
///
/// # Errors
///
/// Returns the backend's error text.
#[server(SearchAccessPrincipals, input = Json)]
pub async fn search_access_principals(query: String) -> Result<PrincipalResults, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let query = query.trim();
        if query.chars().count() < MIN_QUERY_CHARS {
            return Ok(PrincipalResults::default());
        }
        super::backend::backend_or_unwired()
            .search_principals(query)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = query;
        Err(ServerFnError::new(
            "search_access_principals requires the ssr feature",
        ))
    }
}

/// Ask the owners of `resource` for `role`, with a reason.
///
/// # Errors
///
/// Returns an error for a blank or overlong reason, or the backend's error text.
#[server(RequestResourceAccess, input = Json)]
pub async fn request_resource_access(
    resource: ResourceRef,
    role: Role,
    reason: String,
) -> Result<AccessRequestReceipt, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let reason = reason.trim();
        if reason.is_empty() {
            return Err(ServerFnError::new("Add a reason so the owners can decide."));
        }
        if reason.chars().count() > MAX_REASON_CHARS {
            return Err(ServerFnError::new(format!(
                "Keep the reason under {MAX_REASON_CHARS} characters."
            )));
        }
        let id = super::backend::backend_or_unwired()
            .request(&resource, role, reason)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;
        Ok(AccessRequestReceipt {
            id,
            sent_at: chrono::Utc::now().timestamp(),
        })
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = (resource, role, reason);
        Err(ServerFnError::new(
            "request_resource_access requires the ssr feature",
        ))
    }
}

/// Whether the session actor holds `action` on `resource`. False when no backend is wired.
///
/// # Errors
///
/// Returns the backend's error text when the check itself fails.
#[server(CanActOnResource, input = Json)]
pub async fn can_act_on_resource(
    resource: ResourceRef,
    action: String,
) -> Result<bool, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        super::backend::backend_or_unwired()
            .can(&resource, &action)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = (resource, action);
        Err(ServerFnError::new(
            "can_act_on_resource requires the ssr feature",
        ))
    }
}
