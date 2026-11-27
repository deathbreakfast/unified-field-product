//! Reopen sign-in when a session expires partway through.
//!
//! A server fn that finds no signed-in user returns [`unauthenticated_error`]. The
//! page hands any server fn error to [`SessionExpiry::report`]; when it is the
//! unauthenticated error, the shell refreshes the auth session and
//! [`crate::routes::RequireAuthenticated`] reopens sign-in on the same path and query.
//!
//! [`provide_session_expiry`] runs inside `UnifiedFieldShellLayout`, so pages under
//! the shell only call [`use_session_expiry`]:
//!
//! ```rust,ignore
//! use leptos::prelude::*;
//! use leptos::task::spawn_local;
//! use uf_product::use_session_expiry;
//!
//! let expiry = use_session_expiry();
//! let save = move || {
//!     spawn_local(async move {
//!         if let Err(err) = save_task().await {
//!             if !expiry.is_some_and(|e| e.report(&err)) {
//!                 // Show the error as usual.
//!             }
//!         }
//!     });
//! };
//! ```

use leptos::prelude::*;

use crate::{AuthContext, AuthSession};

/// Message carried by [`unauthenticated_error`].
pub const UNAUTHENTICATED_ERROR: &str = "unauthenticated";

/// Message Higgs host helpers return when no user is signed in.
const HIGGS_SIGNED_OUT_MESSAGE: &str = "You must be signed in";

/// The error a server fn returns when the request has no signed-in user.
#[must_use]
pub fn unauthenticated_error() -> ServerFnError {
    ServerFnError::ServerError(UNAUTHENTICATED_ERROR.to_string())
}

/// True when `err` says the request had no signed-in user.
///
/// Matches [`unauthenticated_error`] and the Higgs host "You must be signed in" error.
#[must_use]
pub fn is_unauthenticated_error(err: &ServerFnError) -> bool {
    match err {
        ServerFnError::ServerError(msg) => msg == UNAUTHENTICATED_ERROR,
        ServerFnError::Args(msg) => msg == HIGGS_SIGNED_OUT_MESSAGE,
        _ => false,
    }
}

/// Shell handle that turns an unauthenticated server fn error into a sign-in prompt.
#[derive(Clone, Copy)]
pub struct SessionExpiry {
    expired: RwSignal<bool>,
    reload_token: Option<RwSignal<u64>>,
}

impl SessionExpiry {
    /// Create a handle bound to the [`AuthContext`] in scope, if any.
    #[must_use]
    pub fn new() -> Self {
        let auth = use_context::<AuthContext>();
        let expired = RwSignal::new(false);
        // Every session load sets the signal, even when the value is unchanged. A load
        // that comes back signed in means the report was a false alarm, so drop the
        // flag; otherwise a later deliberate sign-out would reopen sign-in.
        if let Some(session) = auth.as_ref().map(AuthContext::session) {
            Effect::new(move |_| {
                let signed_in = session.with(|s| matches!(s, AuthSession::Authenticated(_)));
                if signed_in && expired.get_untracked() {
                    expired.set(false);
                }
            });
        }
        Self {
            expired,
            reload_token: auth.map(|auth| auth.reload_token()),
        }
    }

    /// Set when a report arrived and sign-in has not reopened yet.
    #[must_use]
    pub fn expired(&self) -> RwSignal<bool> {
        self.expired
    }

    /// Refresh the auth session when `err` is the unauthenticated error.
    ///
    /// Returns `true` when it was, so the caller can skip its own error message.
    pub fn report(&self, err: &ServerFnError) -> bool {
        if !is_unauthenticated_error(err) {
            return false;
        }
        self.expired.set(true);
        if let Some(token) = self.reload_token {
            token.update(|value| *value = value.wrapping_add(1));
        }
        true
    }
}

impl Default for SessionExpiry {
    fn default() -> Self {
        Self::new()
    }
}

/// Provide [`SessionExpiry`] for the current subtree. Call after the auth context.
pub fn provide_session_expiry() -> SessionExpiry {
    let expiry = SessionExpiry::new();
    provide_context(expiry);
    expiry
}

/// Optional access to a provided [`SessionExpiry`].
#[must_use]
pub fn use_session_expiry() -> Option<SessionExpiry> {
    use_context::<SessionExpiry>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unauthenticated_errors_match_happy_path() {
        assert!(is_unauthenticated_error(&unauthenticated_error()));
        assert!(is_unauthenticated_error(&ServerFnError::Args(
            "You must be signed in".into()
        )));
    }

    #[test]
    fn other_errors_do_not_match_sad() {
        assert!(!is_unauthenticated_error(&ServerFnError::new("not found")));
        assert!(!is_unauthenticated_error(&ServerFnError::Args(
            "unauthenticated".into()
        )));
        assert!(!is_unauthenticated_error(&ServerFnError::ServerError(
            "You must be signed in".into()
        )));
    }
}
