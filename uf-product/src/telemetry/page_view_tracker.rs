//! Main-shell page view tracking for `uf_app!` products.

use leptos::prelude::*;
use leptos_router::hooks::use_location;

/// One row in the generated app route table (longest-prefix match).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UfAppRouteEntry {
    pub app_id: &'static str,
    pub app_name: &'static str,
    pub route_prefix: &'static str,
    pub brand_seed: &'static str,
}

/// Resolve the owning app for a pathname using longest-prefix match.
pub fn resolve_app_for_path(path: &str, table: &[UfAppRouteEntry]) -> UfAppRouteEntry {
    let mut best: Option<UfAppRouteEntry> = None;
    for entry in table {
        if path == entry.route_prefix || path.starts_with(&format!("{}/", entry.route_prefix)) {
            let take = best
                .map(|b| entry.route_prefix.len() > b.route_prefix.len())
                .unwrap_or(true);
            if take {
                best = Some(*entry);
            }
        }
    }
    best.unwrap_or(UfAppRouteEntry {
        app_id: "shell",
        app_name: "Shell",
        route_prefix: "/",
        brand_seed: "#1a6f94",
    })
}

/// Tracks navigations under the main app router and emits Spectra page-view telemetry.
///
/// # Examples
///
/// ```rust,ignore
/// use leptos::prelude::*;
/// use uf_product::telemetry::page_view_tracker::{PageViewTracker, UfAppRouteEntry};
///
/// static ROUTES: &[UfAppRouteEntry] = &[UfAppRouteEntry {
///     app_id: "counter",
///     app_name: "Counter",
///     route_prefix: "/counter",
///     brand_seed: "#1a6f94",
/// }];
/// view! { <PageViewTracker routes=ROUTES surface="main".to_string() /> };
/// assert_eq!(ROUTES[0].app_id, "counter");
/// ```
#[component]
pub fn PageViewTracker(
    /// Route paths this applies to.
    routes: &'static [UfAppRouteEntry],
    /// Surface identifier used for tracking.
    #[prop(optional, into)]
    surface: String,
) -> impl IntoView {
    let location = use_location();
    let (last_path, set_last_path) = signal(String::new());

    Effect::new(move |_| {
        let path = location.pathname.get();
        if path.is_empty() {
            return;
        }
        if last_path.get_untracked() == path {
            return;
        }
        let referrer = last_path.get_untracked();
        set_last_path.set(path.clone());

        let entry = resolve_app_for_path(&path, routes);
        let surface = if surface.is_empty() {
            "main_shell".to_string()
        } else {
            surface.clone()
        };

        leptos::task::spawn_local(async move {
            if let Err(err) = record_page_view(
                path,
                entry.app_id.to_string(),
                entry.app_name.to_string(),
                entry.route_prefix.to_string(),
                surface,
                referrer,
                "client_nav".to_string(),
            )
            .await
            {
                leptos::logging::warn!("page view telemetry failed: {err}");
            }
        });
    });

    view! { <></> }
}

#[server(RecordPageView)]
async fn record_page_view(
    /// Pathname that was navigated to.
    path: String,
    /// Id of the app owning the viewed route.
    app_id: String,
    /// Display name of the app owning the viewed route.
    app_name: String,
    /// Route prefix matched for the viewed path.
    route_prefix: String,
    /// Surface the navigation occurred in (e.g. `"main_shell"`).
    surface: String,
    /// Previous pathname, if any, used as the referrer.
    referrer_path: String,
    /// Kind of navigation that triggered this page view (e.g. `"client_nav"`).
    nav_kind: String,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use super::page_views::record_page_view_telemetry;

        let path = truncate_telemetry_field(&path, MAX_TELEMETRY_FIELD_LEN);
        let app_id = truncate_telemetry_field(&app_id, MAX_TELEMETRY_FIELD_LEN);
        let app_name = truncate_telemetry_field(&app_name, MAX_TELEMETRY_FIELD_LEN);
        let route_prefix = truncate_telemetry_field(&route_prefix, MAX_TELEMETRY_FIELD_LEN);
        let surface = truncate_telemetry_field(&surface, MAX_TELEMETRY_FIELD_LEN);
        let referrer_path = truncate_telemetry_field(&referrer_path, MAX_TELEMETRY_FIELD_LEN);
        let nav_kind = truncate_telemetry_field(&nav_kind, MAX_TELEMETRY_FIELD_LEN);

        // Prefer e2e / override viewer keys so PageViewTracker writes match welcome
        // usage queries (Higgs session id is `user:e2e-user`, not the seed viewer).
        let (auth, viewer_key) = match super::usage::resolve_usage_viewer_key().await {
            Some(viewer) => ("authenticated", viewer),
            None => match higgs::Higgs::from_request().await {
                Ok(ctx) => {
                    let auth = if ctx.session_user_id().is_some() {
                        "authenticated"
                    } else {
                        "anonymous"
                    };
                    let viewer_key = ctx
                        .session_user_id()
                        .map(|id| id.to_string())
                        .unwrap_or_else(|| "anonymous".to_string());
                    (auth, viewer_key)
                }
                Err(_) => ("anonymous", "anonymous".to_string()),
            },
        };
        let email_verified = "unknown";

        record_page_view_telemetry(
            &path,
            &app_id,
            &app_name,
            &route_prefix,
            &surface,
            auth,
            email_verified,
            &viewer_key,
            &nav_kind,
            &referrer_path,
            "ok",
            "",
            0,
        );
        if let Some(spectra) = use_context::<std::sync::Arc<spectra::Spectra>>() {
            let _ = spectra.flush_persist().await;
        }
        Ok(())
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = (
            path,
            app_id,
            app_name,
            route_prefix,
            surface,
            referrer_path,
            nav_kind,
        );
        Ok(())
    }
}

/// Cap client-supplied Spectra label/field length (resource abuse / cardinality).
#[cfg(feature = "ssr")]
const MAX_TELEMETRY_FIELD_LEN: usize = 256;

#[cfg(feature = "ssr")]
fn truncate_telemetry_field(value: &str, max_len: usize) -> String {
    if value.len() <= max_len {
        value.to_string()
    } else {
        value.chars().take(max_len).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TABLE: &[UfAppRouteEntry] = &[
        UfAppRouteEntry {
            app_id: "apps",
            app_name: "Apps",
            route_prefix: "/apps",
            brand_seed: "#111111",
        },
        UfAppRouteEntry {
            app_id: "counter",
            app_name: "Counter",
            route_prefix: "/counter",
            brand_seed: "#222222",
        },
        UfAppRouteEntry {
            app_id: "counter-admin",
            app_name: "Counter Admin",
            route_prefix: "/counter/admin",
            brand_seed: "#333333",
        },
    ];

    #[test]
    fn resolve_exact_prefix_match() {
        let entry = resolve_app_for_path("/counter", TABLE);
        assert_eq!(entry.app_id, "counter");
    }

    #[test]
    fn resolve_longest_prefix_wins() {
        let entry = resolve_app_for_path("/counter/admin/settings", TABLE);
        assert_eq!(entry.app_id, "counter-admin");
        assert_eq!(entry.route_prefix, "/counter/admin");
    }

    #[test]
    fn resolve_unknown_falls_back_to_shell() {
        let entry = resolve_app_for_path("/unknown", TABLE);
        assert_eq!(entry.app_id, "shell");
        assert_eq!(entry.route_prefix, "/");
    }

    #[test]
    fn truncate_telemetry_field_caps_length_sad() {
        let long = "a".repeat(300);
        let truncated = truncate_telemetry_field(&long, MAX_TELEMETRY_FIELD_LEN);
        assert_eq!(truncated.len(), MAX_TELEMETRY_FIELD_LEN);
    }

    #[test]
    fn truncate_telemetry_field_keeps_short_happy_path() {
        assert_eq!(
            truncate_telemetry_field("/apps", MAX_TELEMETRY_FIELD_LEN),
            "/apps"
        );
    }
}
