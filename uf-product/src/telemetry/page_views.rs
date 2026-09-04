//! Server-side page view emit helper.

use spectra_core::{try_log_event, try_record_counter};

use super::events::page_view_log_fields;

/// Record one main-shell page view (UC1 + UC3).
#[allow(clippy::too_many_arguments)]
pub fn record_page_view_telemetry(
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
) {
    try_record_counter(
        "uf_apps_page_views",
        &[("app_id", app_id), ("outcome", outcome)],
        1,
    );
    try_log_event(
        "uf_apps_page_view_log",
        &page_view_log_fields(
            path,
            app_id,
            app_name,
            route_prefix,
            surface,
            auth,
            email_verified,
            viewer_key,
            nav_kind,
            referrer_path,
            outcome,
            permission_name,
            role_count,
        ),
    );
}
