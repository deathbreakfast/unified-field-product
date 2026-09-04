//! Runtime permission evaluation plugged in by the host at composition time.

#[cfg(feature = "ssr")]
use std::sync::Arc;

use leptos::prelude::*;

/// Evaluates named permissions for the current request actor.
#[cfg(feature = "ssr")]
#[async_trait::async_trait]
pub trait PermissionBackend: Send + Sync {
    /// Return whether the current actor holds `permission_name`.
    async fn has_permission(&self, permission_name: &str) -> Result<bool, ServerFnError>;
}

/// Install a permission evaluator for this Leptos runtime (host bootstrap).
#[cfg(feature = "ssr")]
pub fn provide_permission_backend(backend: Arc<dyn PermissionBackend>) {
    provide_context(backend);
}

/// Read the installed backend, if any.
#[cfg(feature = "ssr")]
pub fn use_permission_backend() -> Option<Arc<dyn PermissionBackend>> {
    use_context::<Arc<dyn PermissionBackend>>()
}

/// Fail-closed gate for server functions and admin paths.
///
/// # Errors
///
/// Returns [`ServerFnError`] when the backend check fails or the actor lacks
/// `permission_name` (Higgs denial / check-failed payloads).
#[cfg(feature = "ssr")]
pub async fn require_permission(permission_name: &str) -> Result<(), ServerFnError> {
    let allowed = eval_permission_by_name(permission_name)
        .await
        .map_err(|e| {
            ServerFnError::new(higgs::server_runtime::permission_check_failed_payload(
                permission_name,
                &e.to_string(),
            ))
        })?;
    if !allowed {
        return Err(ServerFnError::new(
            higgs::server_runtime::permission_denied_payload(permission_name),
        ));
    }
    Ok(())
}

/// Soft check: returns false when no backend is installed or the check fails.
#[cfg(feature = "ssr")]
pub async fn has_permission(permission_name: &str) -> bool {
    eval_permission_by_name(permission_name)
        .await
        .unwrap_or(false)
}

/// Evaluate a named permission using the installed [`PermissionBackend`].
///
/// Prefer this on the SSR render path so route guards share the same Leptos
/// request context as [`provide_permission_backend`]. Hydrate clients should
/// call [`check_permission_by_name`] (server fn).
///
/// # Errors
///
/// Returns [`ServerFnError`] when the installed backend's check fails. Missing
/// backend or empty name yields `Ok(false)`.
#[cfg(feature = "ssr")]
pub async fn eval_permission_by_name(permission_name: &str) -> Result<bool, ServerFnError> {
    let name = permission_name.trim();
    if name.is_empty() {
        return Ok(false);
    }
    let Some(backend) = use_permission_backend() else {
        return Ok(false);
    };
    backend.has_permission(name).await
}

/// Route-guard helper for hydrate / remote calls: fail closed when unwired.
///
/// # Errors
///
/// Returns [`ServerFnError`] when the backend check itself fails. Missing
/// backend or empty name yields `Ok(false)`.
#[server(CheckPermissionByName)]
pub async fn check_permission_by_name(permission_name: String) -> Result<bool, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return eval_permission_by_name(&permission_name).await;
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = permission_name;
        Ok(false)
    }
}
