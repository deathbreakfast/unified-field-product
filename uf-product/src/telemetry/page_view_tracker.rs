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

/// Title of the page on screen, keyed by its pathname.
///
/// [`PageViewTracker`] provides it; the product app bar writes its last
/// breadcrumb here so each page view can carry the page's title. Leptos context
/// reaches components rendered after the tracker, so mount the tracker before
/// the routes. A title saved for another path is ignored.
#[derive(Clone, Copy, Debug)]
pub struct CurrentPageTitle(RwSignal<Option<(String, String)>>);

impl CurrentPageTitle {
    /// Record `title` as the title of the page at `path`.
    pub fn set(&self, path: String, title: String) {
        self.0.set(Some((path, title)));
    }

    /// Title saved for `path`, if the page at `path` set one.
    #[must_use]
    pub fn title_for(&self, path: &str) -> Option<String> {
        self.0.with_untracked(|current| {
            current
                .as_ref()
                .filter(|(p, _)| p == path)
                .map(|(_, title)| title.clone())
        })
    }
}

/// How long a page view waits for the new page's app bar to set its title.
const TITLE_SETTLE: std::time::Duration = std::time::Duration::from_millis(250);

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
    let page_title = use_context::<CurrentPageTitle>().unwrap_or_else(|| {
        let title = CurrentPageTitle(RwSignal::new(None));
        provide_context(title);
        title
    });

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

        // Wait for the new page's app bar to publish its breadcrumb title.
        set_timeout(
            move || {
                let title = page_title.title_for(&path).unwrap_or_default();
                leptos::task::spawn_local(send_page_view(path, title, entry, surface, referrer));
            },
            TITLE_SETTLE,
        );
    });

    view! { <></> }
}

async fn send_page_view(
    path: String,
    title: String,
    entry: UfAppRouteEntry,
    surface: String,
    referrer: String,
) {
    if let Err(err) = record_page_view(
        path,
        title,
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
}

#[allow(clippy::too_many_arguments)]
#[server(RecordPageView)]
async fn record_page_view(
    /// Pathname that was navigated to.
    path: String,
    /// Page title (last breadcrumb item); empty means use the app name.
    title: String,
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
        let title = page_title_or_app_name(
            &truncate_telemetry_field(&title, MAX_TELEMETRY_FIELD_LEN),
            &app_name,
        );
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
            &title,
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

        // Signed-in viewers also get this page saved as their last page in the app.
        if let Ok(ctx) = crate::ssr::higgs().await {
            if let Some(session_user) = ctx.session_user_id() {
                let user = crate::ssr::session_user_record_id(session_user)?;
                let valence = crate::ssr::valence(&ctx)?;
                super::recent_pages::record_recent_page(
                    &valence,
                    &user,
                    &app_id,
                    &title,
                    &path,
                    chrono::Utc::now(),
                )
                .await
                .map_err(|e| ServerFnError::new(e.to_string()))?;
            }
        }
        Ok(())
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = (
            path,
            title,
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

/// The page title, or `app_name` when the page set none.
#[cfg(feature = "ssr")]
fn page_title_or_app_name(title: &str, app_name: &str) -> String {
    let title = title.trim();
    if title.is_empty() {
        app_name.to_string()
    } else {
        title.to_string()
    }
}

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
    fn page_title_falls_back_to_app_name() {
        assert_eq!(page_title_or_app_name("Task 42", "Magnon"), "Task 42");
        assert_eq!(page_title_or_app_name("  ", "Magnon"), "Magnon");
    }

    #[test]
    fn current_page_title_ignores_other_paths() {
        let owner = Owner::new();
        owner.set();
        let title = CurrentPageTitle(RwSignal::new(None));
        assert_eq!(title.title_for("/a"), None);
        title.set("/a".into(), "Alpha".into());
        assert_eq!(title.title_for("/a").as_deref(), Some("Alpha"));
        assert_eq!(title.title_for("/b"), None);
    }

    #[test]
    fn truncate_telemetry_field_keeps_short_happy_path() {
        assert_eq!(
            truncate_telemetry_field("/apps", MAX_TELEMETRY_FIELD_LEN),
            "/apps"
        );
    }
}
