//! Typed errors for welcome featured-app catalog mutations.

use std::fmt;

/// Library-facing failures from featured catalog `list` / `add` / `remove` / `reorder`.
///
/// Distinct variants keep unknown-app, duplicate, and missing-row cases inspectable
/// before server functions collapse them into `ServerFnError`. Valence failures land
/// in [`Self::Service`].
#[derive(Debug)]
pub enum FeaturedError {
    /// `app_id` is not present in [`uf_product::AppRegistry`].
    UnknownApp {
        /// Requested application id (safe to log).
        app_id: String,
    },
    /// `app_id` is already in the featured catalog.
    Duplicate {
        /// Conflicting application id (safe to log).
        app_id: String,
    },
    /// No featured row for the given bare id or `app_id`.
    NotFound {
        /// Lookup key that missed (safe to log).
        key: String,
    },
    /// Underlying Valence or registry failure.
    Service {
        /// Operation label (e.g. `add`, `remove`, `list`).
        operation: &'static str,
        /// Source error.
        source: anyhow::Error,
    },
}

impl fmt::Display for FeaturedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownApp { app_id } => {
                write!(f, "unknown app_id (not in AppRegistry): {app_id}")
            }
            Self::Duplicate { app_id } => {
                write!(f, "app_id already featured: {app_id}")
            }
            Self::NotFound { key } => write!(f, "featured app not found: {key}"),
            Self::Service { operation, source } => {
                write!(f, "featured catalog {operation} failed: {source}")
            }
        }
    }
}

impl std::error::Error for FeaturedError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Service { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}

impl FeaturedError {
    pub(crate) fn unknown_app(app_id: impl Into<String>) -> Self {
        Self::UnknownApp {
            app_id: app_id.into(),
        }
    }

    pub(crate) fn duplicate(app_id: impl Into<String>) -> Self {
        Self::Duplicate {
            app_id: app_id.into(),
        }
    }

    pub(crate) fn not_found(key: impl Into<String>) -> Self {
        Self::NotFound { key: key.into() }
    }

    pub(crate) fn service(operation: &'static str, source: impl Into<anyhow::Error>) -> Self {
        Self::Service {
            operation,
            source: source.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FeaturedError;
    use std::error::Error;

    #[test]
    fn display_and_source_for_variants() {
        let unknown = FeaturedError::unknown_app("missing");
        assert!(unknown.to_string().contains("missing"));
        assert!(unknown.source().is_none());

        let dup = FeaturedError::duplicate("welcome");
        assert!(dup.to_string().contains("already featured"));
        assert!(dup.source().is_none());

        let missing = FeaturedError::not_found("abc");
        assert!(missing.to_string().contains("not found"));
        assert!(missing.source().is_none());

        let service = FeaturedError::service("add", anyhow::anyhow!("backend down"));
        let msg = service.to_string();
        assert!(msg.contains("add"));
        assert!(msg.contains("backend down"));
        assert!(service.source().is_some());
    }
}
