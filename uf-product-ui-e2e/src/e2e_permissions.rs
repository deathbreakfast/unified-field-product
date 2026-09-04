//! Harness PermissionBackend for product UI e2e (not live Gauge).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use leptos::prelude::*;
use uf_product::permissions::{provide_permission_backend, PermissionBackend};

/// Tower-session key kept for seed JSON / debugging; allow evaluation uses
/// [`E2E_PERMISSION_ALLOW_FLAG`] so `CheckPermissionByName` does not depend on
/// Session extract inside the server-fn path.
pub const E2E_PERMISSION_ALLOW_SESSION_KEY: &str = "uf_e2e_permission_allow";

/// Process-global allow switch for [`E2E_PERMISSION_ALLOW`] (e2e host only).
pub static E2E_PERMISSION_ALLOW_FLAG: AtomicBool = AtomicBool::new(false);

/// Permission name that always denies (deny-gate demo page).
pub const E2E_PERMISSION_DENY: &str = "e2e.permission.deny";

/// Permission name allowed when [`E2E_PERMISSION_ALLOW_FLAG`] is set.
pub const E2E_PERMISSION_ALLOW: &str = "e2e.permission.allow";

struct E2ePermissionBackend;

#[async_trait::async_trait]
impl PermissionBackend for E2ePermissionBackend {
    async fn has_permission(&self, permission_name: &str) -> Result<bool, ServerFnError> {
        match permission_name {
            E2E_PERMISSION_DENY => Ok(false),
            E2E_PERMISSION_ALLOW => Ok(E2E_PERMISSION_ALLOW_FLAG.load(Ordering::SeqCst)),
            _ => Ok(false),
        }
    }
}

/// Install the e2e permission stub into Leptos context (SSR request scope).
pub fn wire_e2e_permissions() {
    provide_permission_backend(Arc::new(E2ePermissionBackend));
}
