//! Typed errors for welcome / product usage queries.

use std::fmt;

/// Failure while querying or aggregating page-view usage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UsageQueryError {
    /// Host did not provide a Spectra handle (or equivalent).
    SpectraUnavailable,
    /// Spectra router query failed.
    QueryFailed {
        /// Safe error class / message from the backend (no PII).
        cause: String,
    },
}

impl fmt::Display for UsageQueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SpectraUnavailable => {
                write!(f, "spectra unavailable for usage query")
            }
            Self::QueryFailed { cause } => {
                write!(f, "usage query failed: {cause}")
            }
        }
    }
}

impl std::error::Error for UsageQueryError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_omits_viewer_key_and_paths() {
        let err = UsageQueryError::QueryFailed {
            cause: "backend timeout".into(),
        };
        let s = err.to_string();
        assert!(s.contains("usage query failed"));
        assert!(s.contains("backend timeout"));
        assert!(!s.contains("viewer_key"));
        assert!(!s.contains('@'));
    }
}
