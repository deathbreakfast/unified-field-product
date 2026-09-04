//! Validate app-relative navigation links for index rows.

use super::WorkspaceSearchError;

/// Accept only same-app relative paths: `/…` without scheme or `//`.
pub(crate) fn validate_relative_link(link: &str) -> Result<(), WorkspaceSearchError> {
    let trimmed = link.trim();
    if trimmed.is_empty() {
        return Err(WorkspaceSearchError::InvalidLink { reason: "empty" });
    }
    if !trimmed.starts_with('/') {
        return Err(WorkspaceSearchError::InvalidLink {
            reason: "not_absolute_path",
        });
    }
    if trimmed.starts_with("//") {
        return Err(WorkspaceSearchError::InvalidLink {
            reason: "protocol_relative",
        });
    }
    let lower = trimmed.to_ascii_lowercase();
    if lower.contains("://") || lower.starts_with("/javascript:") || lower.contains("javascript:") {
        return Err(WorkspaceSearchError::InvalidLink {
            reason: "scheme_or_javascript",
        });
    }
    if trimmed.contains('\\') {
        return Err(WorkspaceSearchError::InvalidLink {
            reason: "backslash",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_app_relative_link_happy() {
        assert!(validate_relative_link("/todo/abc").is_ok());
        assert!(validate_relative_link("/apps").is_ok());
    }

    #[test]
    fn rejects_unsafe_links_sad() {
        assert!(validate_relative_link("https://evil.example/x").is_err());
        assert!(validate_relative_link("//evil.example/x").is_err());
        assert!(validate_relative_link("javascript:alert(1)").is_err());
        assert!(validate_relative_link("").is_err());
        assert!(validate_relative_link("todo/1").is_err());
    }
}
