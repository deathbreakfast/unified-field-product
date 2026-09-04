//! Safe `referer` query parsing for post-auth redirects.

use url::form_urlencoded;

use crate::paths::{AUTH_SIGNIN, AUTH_SIGNUP};

/// Extract the `referer` query parameter from a raw URL search string.
///
/// This helper is typically used by auth pages that need to return the user to the
/// page they were trying to access after sign-in.
///
/// ```rust
/// use uf_product::routes::parse_referer_from_search;
///
/// assert_eq!(
///     parse_referer_from_search("?referer=%2Fcounter%2Fadmin"),
///     Some("/counter/admin".to_string())
/// );
/// assert_eq!(parse_referer_from_search("?foo=bar"), None);
/// ```
pub fn parse_referer_from_search(search: &str) -> Option<String> {
    let trimmed = search.trim_start_matches('?');
    if trimmed.is_empty() {
        return None;
    }

    for (key, value) in form_urlencoded::parse(trimmed.as_bytes()) {
        if key == "referer" {
            return Some(value.into_owned());
        }
    }
    None
}

/// Sanitize and normalize a referer path before redirecting.
///
/// Only safe in-app paths are allowed. Orbital rejects:
///
/// - protocol-relative URLs such as `//evil.example`,
/// - backslash / control-character tricks some browsers treat as `//host`,
/// - absolute URLs smuggled after a leading slash (`/https://…`),
/// - auth/API endpoints that should not be used as a landing page,
/// - `/home` redirects that would create an unnecessary loop.
///
/// Invalid values fall back to `/`.
///
/// ```rust
/// use uf_product::routes::sanitize_referer_path;
///
/// assert_eq!(
///     sanitize_referer_path(Some("/counter/high-scores".to_string())),
///     "/counter/high-scores"
/// );
/// assert_eq!(sanitize_referer_path(Some("//example.com".to_string())), "/");
/// assert_eq!(sanitize_referer_path(Some("/\\evil.example".to_string())), "/");
/// assert_eq!(sanitize_referer_path(Some("/auth/signin".to_string())), "/");
/// ```
pub fn sanitize_referer_path(referer: Option<String>) -> String {
    referer
        .filter(|path| is_safe_in_app_referer_path(path))
        .unwrap_or_else(|| "/".to_string())
}

/// Build an auth route URL that returns the user to `current_path` after success.
///
/// `current_path` is sanitized with [`sanitize_referer_path`] before encoding, so
/// open-redirect inputs never appear raw in the query string.
///
/// ```rust
/// use uf_product::routes::auth_path_with_referer;
///
/// assert_eq!(
///     auth_path_with_referer("/auth/signin", Some("/apps")),
///     "/auth/signin?referer=%2Fapps"
/// );
/// assert_eq!(
///     auth_path_with_referer("/auth/signin", Some("//evil.example")),
///     "/auth/signin?referer=%2F"
/// );
/// ```
pub fn auth_path_with_referer(auth_path: &str, current_path: Option<&str>) -> String {
    let referer = sanitize_referer_path(current_path.map(str::to_owned));
    let mut serializer = form_urlencoded::Serializer::new(String::new());
    serializer.append_pair("referer", &referer);
    format!("{auth_path}?{}", serializer.finish())
}

/// [`AUTH_SIGNIN`](crate::paths::AUTH_SIGNIN) with a sanitized `referer` query.
#[must_use]
pub fn auth_signin_href(current_path: Option<&str>) -> String {
    auth_path_with_referer(AUTH_SIGNIN, current_path)
}

/// [`AUTH_SIGNUP`](crate::paths::AUTH_SIGNUP) with a sanitized `referer` query.
#[must_use]
pub fn auth_signup_href(current_path: Option<&str>) -> String {
    auth_path_with_referer(AUTH_SIGNUP, current_path)
}

/// True when `path` is a same-origin absolute path suitable for post-auth redirect.
fn is_safe_in_app_referer_path(path: &str) -> bool {
    if !path.starts_with('/') || path.starts_with("//") {
        return false;
    }
    // `/\evil.example` is protocol-relative in some browsers.
    if path.contains('\\') {
        return false;
    }
    // Reject ASCII controls / whitespace (tab, CR, LF, NUL, …).
    if path.bytes().any(|b| b <= 0x20 || b == 0x7f) {
        return false;
    }
    // Reject absolute URLs smuggled as paths (`/https://evil.example`).
    if path.contains("://") {
        return false;
    }
    if path.starts_with("/auth/") || path.starts_with("/api/") {
        return false;
    }
    if path == "/home" || path == "/home/" {
        return false;
    }
    true
}

#[cfg(test)]
mod referer_tests {
    use super::{
        auth_path_with_referer, auth_signin_href, auth_signup_href, parse_referer_from_search,
        sanitize_referer_path,
    };

    #[test]
    fn parse_referer_decodes_query_value_happy_path() {
        assert_eq!(
            parse_referer_from_search("?referer=%2Fcounter%2Fadmin"),
            Some("/counter/admin".to_string())
        );
    }

    #[test]
    fn parse_referer_missing_or_empty_none_sad() {
        assert_eq!(parse_referer_from_search("?foo=bar"), None);
        assert_eq!(parse_referer_from_search(""), None);
    }

    #[test]
    fn sanitize_referer_keeps_safe_path_happy_path() {
        assert_eq!(
            sanitize_referer_path(Some("/counter/high-scores".to_string())),
            "/counter/high-scores"
        );
    }

    #[test]
    fn sanitize_referer_rejects_unsafe_paths_sad() {
        assert_eq!(
            sanitize_referer_path(Some("//example.com".to_string())),
            "/"
        );
        assert_eq!(sanitize_referer_path(Some("/auth/signin".to_string())), "/");
        assert_eq!(sanitize_referer_path(Some("/api/x".to_string())), "/");
        assert_eq!(sanitize_referer_path(Some("/home".to_string())), "/");
        assert_eq!(sanitize_referer_path(None), "/");
    }

    #[test]
    fn sanitize_referer_rejects_backslash_and_control_bypasses_sad() {
        assert_eq!(
            sanitize_referer_path(Some("/\\evil.example".to_string())),
            "/"
        );
        assert_eq!(
            sanitize_referer_path(Some("/\tevil.example".to_string())),
            "/"
        );
        assert_eq!(
            sanitize_referer_path(Some("/https://evil.example".to_string())),
            "/"
        );
        assert_eq!(
            sanitize_referer_path(Some("/counter/admin\n".to_string())),
            "/"
        );
    }

    #[test]
    fn auth_path_with_referer_encodes_safe_path_happy_path() {
        assert_eq!(
            auth_path_with_referer("/auth/signin", Some("/apps")),
            "/auth/signin?referer=%2Fapps"
        );
        assert_eq!(
            auth_signin_href(Some("/welcome?tab=1")),
            "/auth/signin?referer=%2Fwelcome%3Ftab%3D1"
        );
        assert_eq!(
            auth_signup_href(Some("/gate/auth-required")),
            "/auth/signup?referer=%2Fgate%2Fauth-required"
        );
    }

    #[test]
    fn auth_path_with_referer_rejects_evil_to_safe_query_sad() {
        let href = auth_path_with_referer("/auth/signin", Some("//evil.example"));
        assert_eq!(href, "/auth/signin?referer=%2F");
        assert!(!href.contains("evil"));
        assert_eq!(
            auth_signin_href(Some("/auth/signin")),
            "/auth/signin?referer=%2F"
        );
    }
}
