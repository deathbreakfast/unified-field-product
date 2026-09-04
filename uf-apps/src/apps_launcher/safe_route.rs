//! App-relative route guards for launcher navigation.

/// Returns `path` when it is safe to hand to the client router as an in-app target.
///
/// Accepts paths that start with a single `/` (for example `/welcome`).
/// Rejects scheme-relative URLs (`//evil.example`), absolute URLs, and empty strings.
pub fn safe_app_route_path(path: &str) -> Option<&str> {
    let p = path.trim();
    if p.is_empty() {
        return None;
    }
    if !p.starts_with('/') || p.starts_with("//") {
        return None;
    }
    if p.contains("://") {
        return None;
    }
    Some(p)
}

#[cfg(test)]
mod tests {
    use super::safe_app_route_path;

    #[test]
    fn accepts_app_relative_paths() {
        assert_eq!(safe_app_route_path("/welcome"), Some("/welcome"));
        assert_eq!(safe_app_route_path("/apps"), Some("/apps"));
        assert_eq!(safe_app_route_path("  /counter  "), Some("/counter"));
    }

    #[test]
    fn rejects_open_redirect_shapes() {
        assert_eq!(safe_app_route_path("//evil.example"), None);
        assert_eq!(safe_app_route_path("https://evil.example"), None);
        assert_eq!(safe_app_route_path("http://evil.example/x"), None);
        assert_eq!(safe_app_route_path("/https://evil.example"), None);
        assert_eq!(safe_app_route_path(""), None);
        assert_eq!(safe_app_route_path("welcome"), None);
    }
}
