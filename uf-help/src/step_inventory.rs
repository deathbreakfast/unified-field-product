//! Help step inventory collected from `#[help_spotlight_step]`.
//!
//! Each [`HelpStepDescriptor`] is submitted through the `inventory` crate at link
//! time. App crates keep steps in a `help_steps` module and expose an empty
//! `ensure_help_steps_linked()` that references every step component so the linker
//! retains submissions (see `uf_apps::ensure_help_linked`).
//!
//! [`collect_help_steps_for_route`] filters by [`route_matches`]: exact pathname
//! equality, or a segment pattern where `:param` matches one non-empty path
//! segment (for example `"/apps/:app_name"` or `"/boson/tasks/:task_name/config"`).
//! [`inventory_route_keys_for_pathname`] returns the pattern keys used when
//! reading or writing Valence visit rows (not the live browser slug).

use leptos::prelude::AnyView;
use uf_product::primitives::PopoverPosition;

/// One spotlight tour step registered by product UI authors.
///
/// `feature_highlight` is the durable identity for “seen”. Adding a new key later
/// shows only that step to users who already finished older ones on the route.
pub struct HelpStepDescriptor {
    /// Route this step belongs to.
    ///
    /// Usually an exact pathname (for example `"/apps"` or `"/welcome"`).
    /// Patterns may include `:param` segments that match one non-empty path
    /// segment each (for example `"/apps/:app_name"` or
    /// `"/boson/tasks/:task_name/config"`).
    pub route: &'static str,
    /// Stable per-step identity within the route.
    pub feature_highlight: &'static str,
    /// Human-readable panel header (defaults to `feature_highlight` when the
    /// macro omits `title`).
    pub title: &'static str,
    /// Optional DOM element id for the spotlight cutout.
    ///
    /// When omitted or empty, Orbital centers the panel in the viewport (no
    /// cutout target).
    pub spotlight: Option<&'static str>,
    /// Sort order within the route (lower first).
    pub order: u16,
    /// Popover placement relative to the spotlight anchor (default: Top).
    pub position: Option<PopoverPosition>,
    /// Renders the step body as an [`AnyView`].
    pub render: fn() -> AnyView,
}

inventory::collect!(HelpStepDescriptor);

/// Collect every registered help step, sorted by [`HelpStepDescriptor::order`].
#[must_use]
pub fn collect_help_steps() -> Vec<&'static HelpStepDescriptor> {
    let mut items: Vec<&'static HelpStepDescriptor> =
        inventory::iter::<HelpStepDescriptor>.into_iter().collect();
    items.sort_by_key(|d| d.order);
    items
}

/// Whether inventory `pattern` applies to browser `pathname`.
///
/// Exact equality always matches. The Permission index inventory key
/// [`/permission/permissions`] also matches bare [`/permission`] (same page,
/// two URLs). Otherwise, when `pattern` contains at least one `:param`
/// segment, each literal segment must equal the corresponding pathname
/// segment and each `:param` must match one non-empty segment (no `/`).
/// Segment counts must match after trimming a trailing `/`.
#[must_use]
pub fn route_matches(pattern: &str, pathname: &str) -> bool {
    if pattern == pathname {
        return true;
    }
    if matches_permission_index_alias(pattern, pathname) {
        return true;
    }
    if !pattern.contains(':') {
        return false;
    }
    matches_param_pattern(pattern, pathname)
}

/// Inventory `/permission/permissions` also covers bare `/permission`.
fn matches_permission_index_alias(pattern: &str, pathname: &str) -> bool {
    if pattern != "/permission/permissions" {
        return false;
    }
    let path = pathname.trim_end_matches('/');
    path == "/permission" || path == "/permission/permissions"
}

fn path_segments(path: &str) -> Vec<&str> {
    path.trim_end_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .collect()
}

fn matches_param_pattern(pattern: &str, pathname: &str) -> bool {
    let pat = path_segments(pattern);
    let live = path_segments(pathname);
    if pat.len() != live.len() || pat.is_empty() {
        return false;
    }
    pat.iter().zip(live.iter()).all(|(p, l)| {
        if let Some(name) = p.strip_prefix(':') {
            !name.is_empty() && !l.is_empty() && !l.contains('/')
        } else {
            p == l
        }
    })
}

/// Collect help steps whose [`HelpStepDescriptor::route`] matches `pathname`.
#[must_use]
pub fn collect_help_steps_for_route(pathname: &str) -> Vec<&'static HelpStepDescriptor> {
    collect_help_steps()
        .into_iter()
        .filter(|d| route_matches(d.route, pathname))
        .collect()
}

/// Distinct inventory route keys that apply to `pathname` (exact or pattern).
///
/// Used when reading/writing Valence visits so rows keyed by a pattern such as
/// `"/apps/:app_name"` are found while browsing `/apps/{slug}`.
#[must_use]
pub fn inventory_route_keys_for_pathname(pathname: &str) -> Vec<&'static str> {
    let mut keys: Vec<&'static str> = collect_help_steps_for_route(pathname)
        .into_iter()
        .map(|d| d.route)
        .collect();
    keys.sort_unstable();
    keys.dedup();
    if keys.is_empty() {
        // Preserve legacy callers that store the raw pathname.
        return Vec::new();
    }
    keys
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collect_help_steps_is_sorted_by_order() {
        let items = collect_help_steps();
        let mut prev = 0u16;
        for item in &items {
            assert!(item.order >= prev);
            prev = item.order;
            assert!(!item.route.is_empty());
            assert!(!item.feature_highlight.is_empty());
        }
    }

    #[test]
    fn route_matches_exact() {
        assert!(route_matches("/apps", "/apps"));
        assert!(route_matches("/welcome", "/welcome"));
        assert!(!route_matches("/apps", "/apps/welcome"));
        assert!(!route_matches("/welcome", "/apps"));
    }

    #[test]
    fn route_matches_apps_app_name_pattern() {
        assert!(route_matches("/apps/:app_name", "/apps/welcome"));
        assert!(route_matches("/apps/:app_name", "/apps/apps"));
        assert!(route_matches("/apps/:app_name", "/apps/foo"));
        assert!(!route_matches("/apps/:app_name", "/apps"));
        assert!(!route_matches("/apps/:app_name", "/apps/"));
        assert!(!route_matches("/apps/:app_name", "/apps/foo/bar"));
        assert!(!route_matches("/apps/:app_name", "/welcome"));
    }

    #[test]
    fn route_matches_multi_segment_param_patterns() {
        assert!(route_matches(
            "/boson/tasks/:task_name",
            "/boson/tasks/send_mail"
        ));
        assert!(route_matches(
            "/boson/tasks/:task_name/config",
            "/boson/tasks/send_mail/config"
        ));
        assert!(route_matches("/boson/runs/:id", "/boson/runs/abc-123"));
        assert!(!route_matches(
            "/boson/tasks/:task_name",
            "/boson/tasks/send_mail/config"
        ));
        assert!(!route_matches(
            "/boson/tasks/:task_name/config",
            "/boson/tasks/send_mail"
        ));
        assert!(!route_matches("/boson/runs/:id", "/boson/runs"));
        assert!(!route_matches("/boson/runs/:id", "/boson/runs/a/b"));
        assert!(!route_matches("/boson/tasks/:task_name", "/boson/tasks/"));
    }

    #[test]
    fn route_matches_rejects_empty_param_name() {
        assert!(!route_matches("/foo/:/bar", "/foo/x/bar"));
    }

    #[test]
    fn route_matches_permission_index_alias() {
        assert!(route_matches(
            "/permission/permissions",
            "/permission/permissions"
        ));
        assert!(route_matches("/permission/permissions", "/permission"));
        assert!(route_matches("/permission/permissions", "/permission/"));
        assert!(!route_matches(
            "/permission/permissions",
            "/permission/permissions/abc"
        ));
        assert!(!route_matches("/permission", "/permission/permissions"));
        assert!(route_matches(
            "/permission/permissions/:id",
            "/permission/permissions/abc"
        ));
        assert!(!route_matches("/permission/permissions/:id", "/permission"));
    }
}
