//! Permission-denied toast bus and server-fn error parsing.
//!
//! Shell layouts subscribe to [`PermissionToastBus`]. When a server fn fails with a
//! `permission_denied::…` or `permission_check_failed::…` message, call
//! [`report_server_fn_error`] (or [`report_server_fn_error_with_bus`]) so the toast
//! surfaces a typed [`PermissionServerError`] instead of a raw string.
//!
//! # Concern → API
//!
//! | Concern | API |
//! |---------|-----|
//! | Parse a server-fn error string | [`parse_permission_server_error`], [`PermissionServerError`] |
//! | Emit a toast from shell layout | [`provide_permission_toast_bus`], [`PermissionToastBus`] |
//! | Report from a server-fn `Err` | [`report_server_fn_error`] |

use leptos::prelude::*;

const PERMISSION_DENIED_PREFIX: &str = "permission_denied::";
const PERMISSION_CHECK_FAILED_PREFIX: &str = "permission_check_failed::";

/// Classified permission failure carried in a server-fn error string.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PermissionServerError {
    /// Caller lacks the named permission.
    Denied {
        /// Canonical permission name (e.g. `counter.admin.set`).
        permission: String,
    },
    /// Permission check itself failed (misconfiguration / backend).
    CheckFailed {
        /// Permission that was being checked.
        permission: String,
        /// Safe detail for operators / toast body.
        details: String,
    },
}

/// Payload for a permission-denied toast.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PermissionToastRequest {
    /// Permission name shown to the user.
    pub permission: String,
    /// Human-readable toast body.
    pub description: String,
}

/// Write handle for permission toasts (provided near the shell root).
#[derive(Clone, Copy)]
pub struct PermissionToastBus {
    set_request: WriteSignal<Option<PermissionToastRequest>>,
}

impl PermissionToastBus {
    /// Replace the pending toast request (latest wins).
    pub fn emit(&self, request: PermissionToastRequest) {
        self.set_request.set(Some(request));
    }
}

/// Provide a toast bus in Leptos context and return the handle.
pub fn provide_permission_toast_bus(
    set_request: WriteSignal<Option<PermissionToastRequest>>,
) -> PermissionToastBus {
    let bus = PermissionToastBus { set_request };
    provide_context(bus);
    bus
}

/// Read the toast bus from context, if provided.
#[must_use]
pub fn use_permission_toast_bus() -> Option<PermissionToastBus> {
    use_context::<PermissionToastBus>()
}

/// Parse a server-fn error message into a [`PermissionServerError`], if it uses
/// the `permission_denied::` / `permission_check_failed::` prefixes.
#[must_use]
pub fn parse_permission_server_error(message: &str) -> Option<PermissionServerError> {
    let denied_message = message.strip_prefix(PERMISSION_DENIED_PREFIX).map_or_else(
        || {
            message
                .find(PERMISSION_DENIED_PREFIX)
                .map(|idx| &message[idx + PERMISSION_DENIED_PREFIX.len()..])
        },
        Some,
    );

    if let Some(permission) = denied_message {
        let permission = permission.trim();
        if !permission.is_empty() {
            return Some(PermissionServerError::Denied {
                permission: permission.to_string(),
            });
        }
    }

    let check_failed_message = message
        .strip_prefix(PERMISSION_CHECK_FAILED_PREFIX)
        .map_or_else(
            || {
                message
                    .find(PERMISSION_CHECK_FAILED_PREFIX)
                    .map(|idx| &message[idx + PERMISSION_CHECK_FAILED_PREFIX.len()..])
            },
            Some,
        );

    if let Some(rest) = check_failed_message {
        let mut parts = rest.splitn(2, "::");
        let permission = parts.next().unwrap_or_default().trim().to_string();
        let details = parts.next().unwrap_or_default().trim().to_string();
        if !permission.is_empty() {
            return Some(PermissionServerError::CheckFailed {
                permission,
                details,
            });
        }
    }

    None
}

/// Report a server-fn error to the toast bus when it encodes a permission failure.
///
/// Returns `true` when the message was recognized and a toast was emitted.
pub fn report_server_fn_error(error: &ServerFnError) -> bool {
    report_server_fn_error_with_bus(use_permission_toast_bus(), error)
}

/// Like [`report_server_fn_error`], but with an explicit bus (useful in tests).
pub fn report_server_fn_error_with_bus(
    bus: Option<PermissionToastBus>,
    error: &ServerFnError,
) -> bool {
    let message = error.to_string();
    let Some(parsed) = parse_permission_server_error(&message) else {
        return false;
    };

    if let Some(bus) = bus {
        let request = match parsed {
            PermissionServerError::Denied { permission } => PermissionToastRequest {
                permission,
                description: "You do not have permission to perform this action.".to_string(),
            },
            PermissionServerError::CheckFailed {
                permission,
                details,
            } => PermissionToastRequest {
                permission,
                description: format!("Unable to verify permission: {details}"),
            },
        };
        bus.emit(request);
        return true;
    }

    false
}
