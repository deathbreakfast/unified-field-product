//! Typed errors for workspace content index write/query.

use std::fmt;

/// Failure from workspace search writer or query.
#[derive(Debug)]
pub enum WorkspaceSearchError {
    /// No authenticated user actor on the Valence handle.
    Unauthenticated,
    /// Query was invoked with System (or other non-user) actor.
    InvalidActor {
        /// Stable reason token for logs / mapping.
        reason: &'static str,
    },
    /// Query text failed validation.
    InvalidQuery {
        /// Stable reason token.
        reason: &'static str,
    },
    /// `link` is not a safe app-relative path.
    InvalidLink {
        /// Stable reason token.
        reason: &'static str,
    },
    /// Persistence or Valence failure (no row payloads).
    Valence {
        /// High-level operation name.
        operation: &'static str,
        /// Safe cause class / message without searchable text.
        message: String,
    },
    /// Writer-specific failure.
    Write {
        /// `upsert` or `delete`.
        operation: &'static str,
        /// Source table id (not title text).
        source_table: String,
        /// Safe cause.
        message: String,
    },
}

impl WorkspaceSearchError {
    /// Low-cardinality class for telemetry / retries.
    #[must_use]
    pub fn error_class(&self) -> &'static str {
        match self {
            Self::Unauthenticated => "unauthenticated",
            Self::InvalidActor { .. } => "invalid_actor",
            Self::InvalidQuery { .. } => "invalid_query",
            Self::InvalidLink { .. } => "invalid_link",
            Self::Valence { .. } => "valence",
            Self::Write { .. } => "write",
        }
    }

    /// Whether a retry might help.
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Valence { .. } | Self::Write { .. })
    }
}

impl fmt::Display for WorkspaceSearchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unauthenticated => write!(f, "workspace search requires an authenticated user"),
            Self::InvalidActor { reason } => {
                write!(f, "workspace search rejected actor ({reason})")
            }
            Self::InvalidQuery { reason } => {
                write!(f, "invalid workspace search query ({reason})")
            }
            Self::InvalidLink { reason } => {
                write!(f, "invalid workspace search link ({reason})")
            }
            Self::Valence { operation, message } => {
                write!(f, "workspace search {operation} failed: {message}")
            }
            Self::Write {
                operation,
                source_table,
                message,
            } => write!(
                f,
                "workspace search {operation} for source_table `{source_table}` failed: {message}"
            ),
        }
    }
}

impl std::error::Error for WorkspaceSearchError {}

impl From<valence::Error> for WorkspaceSearchError {
    fn from(err: valence::Error) -> Self {
        Self::Valence {
            operation: "valence",
            message: err.to_string(),
        }
    }
}
