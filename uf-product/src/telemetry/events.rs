//! UC3 field builders for orbital shell page views.

use serde_json::{json, Value};

#[allow(clippy::too_many_arguments)]
pub fn page_view_log_fields(
    path: &str,
    app_id: &str,
    app_name: &str,
    route_prefix: &str,
    surface: &str,
    auth: &str,
    email_verified: &str,
    viewer_key: &str,
    nav_kind: &str,
    referrer_path: &str,
    outcome: &str,
    permission_name: &str,
    role_count: i64,
) -> Value {
    json!({
        "path": path,
        "app_id": app_id,
        "app_name": app_name,
        "route_prefix": route_prefix,
        "surface": surface,
        "auth": auth,
        "email_verified": email_verified,
        "viewer_key": viewer_key,
        "nav_kind": nav_kind,
        "referrer_path": referrer_path,
        "outcome": outcome,
        "permission_name": permission_name,
        "role_count": role_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_view_log_fields_shape() {
        let v = page_view_log_fields(
            "/counter",
            "counter",
            "Counter",
            "/counter",
            "main_shell",
            "authenticated",
            "unknown",
            "user-1",
            "client_nav",
            "/apps",
            "ok",
            "",
            0,
        );
        assert_eq!(v["path"], "/counter");
        assert_eq!(v["app_id"], "counter");
        assert_eq!(v["surface"], "main_shell");
        assert_eq!(v["auth"], "authenticated");
        assert_eq!(v["nav_kind"], "client_nav");
        assert_eq!(v["outcome"], "ok");
        assert_eq!(v["role_count"], 0);
    }
}
