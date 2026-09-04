#![allow(clippy::too_many_arguments)]

use spectra::spectra_schema;

spectra_schema! {
    UfAppsPageViewLog {
        store: "uf",
        table: "uf_apps_page_view_log",
        version: "0.1.0",
        description: "Main-shell page view events across uf_app! products.",
        fields: [
            path: {
                r#type: String,
                classification: { pii: false, safe_for_console: true },
            },
            app_id: {
                r#type: String,
                classification: { pii: false, safe_for_console: true },
            },
            app_name: {
                r#type: String,
                classification: { pii: false, safe_for_console: true },
            },
            route_prefix: {
                r#type: String,
                classification: { pii: false, safe_for_console: true },
            },
            surface: {
                r#type: String,
                classification: { pii: false, safe_for_console: true },
            },
            auth: {
                r#type: String,
                classification: { pii: false, safe_for_console: true },
            },
            email_verified: {
                r#type: String,
                classification: { pii: false, safe_for_console: true },
            },
            viewer_key: {
                r#type: String,
                classification: { pii: true, safe_for_console: false },
            },
            nav_kind: {
                r#type: String,
                classification: { pii: false, safe_for_console: true },
            },
            referrer_path: {
                r#type: String,
                classification: { pii: false, safe_for_console: true },
            },
            outcome: {
                r#type: String,
                classification: { pii: false, safe_for_console: true },
            },
            permission_name: {
                r#type: String,
                classification: { pii: false, safe_for_console: true },
            },
            role_count: {
                r#type: i64,
                classification: { pii: false, safe_for_console: true },
            },
        ],
    }
}
