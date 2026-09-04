//! Typed errors for Help server and service paths.
//!
//! Public server functions return [`leptos::prelude::ServerFnError`]. SSR paths map
//! [`HelpError`] with [`HelpError::into_server_fn_error`], which wraps
//! [`ServerFnError::ServerError`](leptos::prelude::ServerFnError) and copies only
//! the [`Display`](std::fmt::Display) string. Clients cannot recover the enum variant
//! from the wire; match on message text only for diagnostics, not for control flow.

use leptos::prelude::ServerFnError;
use thiserror::Error;

/// Failures from Help visits, reports, and GitHub filing.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum HelpError {
    /// Caller has no session where Valence ownership is required.
    #[error("unauthenticated: {0}")]
    Unauthenticated(&'static str),
    /// Valence privacy denied the operation.
    #[error("forbidden: {0}")]
    Forbidden(&'static str),
    /// Route string failed validation.
    #[error("invalid route: {0}")]
    InvalidRoute(&'static str),
    /// Feature highlight string failed validation.
    #[error("invalid highlight: {0}")]
    InvalidHighlight(&'static str),
    /// Storage / Valence I/O failure (no payload bodies).
    #[error("storage: {0}")]
    Storage(String),
    /// Rare dual-write merge conflict.
    #[error("merge conflict: {0}")]
    MergeConflict(&'static str),
    /// Form field validation failed.
    #[error("validation: {0}")]
    Validation(String),
    /// Submit throttle exceeded.
    #[error("rate limited; retry after {retry_after_secs}s")]
    RateLimited {
        /// Suggested wait before retry.
        retry_after_secs: u64,
    },
    /// GitHub API failure (status class only).
    #[error("github upstream: {0}")]
    GitHubUpstream(String),
    /// Host secret / repository configuration missing.
    #[error("misconfigured: {0}")]
    Misconfigured(&'static str),
}

impl HelpError {
    /// Map into a Leptos [`ServerFnError`] for server fn returns.
    ///
    /// The variant is flattened to a single server error string; do not assume
    /// structured deserialization on the client.
    #[must_use]
    pub fn into_server_fn_error(self) -> ServerFnError {
        ServerFnError::ServerError(self.to_string())
    }
}
