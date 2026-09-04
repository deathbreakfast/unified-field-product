//! Client-side open-path sanitize (mirrors core `sanitize_notification_url`).

/// Inbox fallback when a notification URL is missing or rejected.
pub(crate) const NOTIFICATIONS_PATH: &str = "/notifications";

/// Keep same-origin relative paths; reject open-redirect and auth/API loops.
///
/// Mirrors `uf_notifications_core::sanitize_notification_url` / `is_safe_notification_path`
/// (backslash, controls, `://` smuggling, `//`, `/auth/`, `/api/`, `/home`).
pub(crate) fn safe_notification_nav_url(url: Option<&str>) -> &str {
    match url {
        Some(path) if is_safe_notification_nav_path(path) => path,
        _ => NOTIFICATIONS_PATH,
    }
}

fn is_safe_notification_nav_path(path: &str) -> bool {
    if !path.starts_with('/') || path.starts_with("//") {
        return false;
    }
    if path.contains('\\') {
        return false;
    }
    if path.bytes().any(|b| b <= 0x20 || b == 0x7f) {
        return false;
    }
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
mod tests {
    use super::{safe_notification_nav_url, NOTIFICATIONS_PATH};

    #[test]
    fn safe_notification_nav_url_keeps_safe_paths_happy() {
        assert_eq!(
            safe_notification_nav_url(Some("/high-scores")),
            "/high-scores"
        );
    }

    #[test]
    fn safe_notification_nav_url_rejects_backslash_control_and_url_smuggle_sad() {
        assert_eq!(
            safe_notification_nav_url(Some("/\\evil.example")),
            NOTIFICATIONS_PATH
        );
        assert_eq!(
            safe_notification_nav_url(Some("/\tevil.example")),
            NOTIFICATIONS_PATH
        );
        assert_eq!(
            safe_notification_nav_url(Some("/https://evil.example")),
            NOTIFICATIONS_PATH
        );
        assert_eq!(
            safe_notification_nav_url(Some("//evil.example")),
            NOTIFICATIONS_PATH
        );
        assert_eq!(
            safe_notification_nav_url(Some("/auth/signin")),
            NOTIFICATIONS_PATH
        );
        assert_eq!(safe_notification_nav_url(None), NOTIFICATIONS_PATH);
    }
}
