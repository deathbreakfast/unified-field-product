//! Visit DTOs and pure pending / merge helpers.

use serde::{Deserialize, Serialize};

/// One visit record (Valence row or localStorage mirror).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelpVisitRecord {
    /// Route path this visit belongs to.
    pub route: String,
    /// Stable feature highlight key.
    pub feature_highlight: String,
    /// Optional spotlight DOM id snapshot.
    pub spotlight: Option<String>,
    /// When true, the step is pending again until marked seen.
    pub replay: bool,
}

/// Step key used when marking seen or computing pending.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelpStepKey {
    /// Route path.
    pub route: String,
    /// Feature highlight key.
    pub feature_highlight: String,
    /// Optional spotlight id to store on the visit row.
    pub spotlight: Option<String>,
}

/// Pending = inventory step with no visit row, or visit with `replay == true`.
#[must_use]
pub fn compute_pending<'a>(
    inventory: &[&'a crate::HelpStepDescriptor],
    visits: &[HelpVisitRecord],
) -> Vec<&'a crate::HelpStepDescriptor> {
    inventory
        .iter()
        .copied()
        .filter(|step| {
            match visits
                .iter()
                .find(|v| v.route == step.route && v.feature_highlight == step.feature_highlight)
            {
                None => true,
                Some(v) => v.replay,
            }
        })
        .collect()
}

/// Set `replay = true` for visits whose stored route matches `pathname`.
///
/// `pathname` is the live browser path; visit rows may store an inventory
/// pattern such as `"/apps/:app_name"`.
#[must_use]
pub fn apply_replay_for_route(visits: &[HelpVisitRecord], pathname: &str) -> Vec<HelpVisitRecord> {
    visits
        .iter()
        .map(|v| {
            let mut next = v.clone();
            if crate::route_matches(&next.route, pathname) {
                next.replay = true;
            }
            next
        })
        .collect()
}

/// Merge local anon visits into server visits for the same user.
///
/// Existing server rows win on `(route, feature_highlight)`. Local-only rows are
/// appended so first authenticated write does not drop device progress.
#[must_use]
pub fn merge_local_into_server(
    server: &[HelpVisitRecord],
    local: &[HelpVisitRecord],
) -> Vec<HelpVisitRecord> {
    let mut out = server.to_vec();
    for local_row in local {
        let exists = out.iter().any(|s| {
            s.route == local_row.route && s.feature_highlight == local_row.feature_highlight
        });
        if !exists {
            out.push(local_row.clone());
        }
    }
    out
}

/// Parse Valence `replay` string (`"true"` / `"false"`) into a bool.
#[must_use]
pub fn replay_from_stored(value: &str) -> bool {
    value.eq_ignore_ascii_case("true")
}

/// Encode a bool as the Valence string form.
#[must_use]
pub fn replay_to_stored(replay: bool) -> String {
    if replay {
        "true".to_string()
    } else {
        "false".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use leptos::prelude::*;

    fn desc(
        route: &'static str,
        feature_highlight: &'static str,
        order: u16,
    ) -> crate::HelpStepDescriptor {
        crate::HelpStepDescriptor {
            route,
            feature_highlight,
            title: feature_highlight,
            spotlight: None,
            order,
            position: None,
            render: || ().into_any(),
        }
    }

    #[test]
    fn compute_pending_missing_and_replay() {
        let a = desc("/apps", "a", 1);
        let b = desc("/apps", "b", 2);
        let c = desc("/apps", "c", 3);
        let inventory = [&a, &b, &c];
        let visits = vec![
            HelpVisitRecord {
                route: "/apps".into(),
                feature_highlight: "a".into(),
                spotlight: None,
                replay: false,
            },
            HelpVisitRecord {
                route: "/apps".into(),
                feature_highlight: "b".into(),
                spotlight: None,
                replay: true,
            },
        ];
        let pending = compute_pending(&inventory, &visits);
        let keys: Vec<_> = pending.iter().map(|d| d.feature_highlight).collect();
        assert_eq!(keys, ["b", "c"]);
    }

    #[test]
    fn compute_pending_empty_inventory() {
        let pending = compute_pending(&[], &[]);
        assert!(pending.is_empty());
    }

    #[test]
    fn apply_replay_is_route_scoped() {
        let visits = vec![
            HelpVisitRecord {
                route: "/apps".into(),
                feature_highlight: "a".into(),
                spotlight: None,
                replay: false,
            },
            HelpVisitRecord {
                route: "/welcome".into(),
                feature_highlight: "w".into(),
                spotlight: None,
                replay: false,
            },
        ];
        let next = apply_replay_for_route(&visits, "/apps");
        assert!(next[0].replay);
        assert!(!next[1].replay);
    }

    #[test]
    fn pattern_route_visit_counts_for_every_app_slug() {
        let step = desc("/apps/:app_name", "app-overview-more-information", 10);
        let visits = vec![HelpVisitRecord {
            route: "/apps/:app_name".into(),
            feature_highlight: "app-overview-more-information".into(),
            spotlight: None,
            replay: false,
        }];
        assert!(compute_pending(&[&step], &visits).is_empty());
        assert!(crate::route_matches("/apps/:app_name", "/apps/welcome"));
        assert!(crate::route_matches("/apps/:app_name", "/apps/apps"));
    }

    #[test]
    fn merge_prefers_server_rows() {
        let server = vec![HelpVisitRecord {
            route: "/apps".into(),
            feature_highlight: "a".into(),
            spotlight: None,
            replay: false,
        }];
        let local = vec![
            HelpVisitRecord {
                route: "/apps".into(),
                feature_highlight: "a".into(),
                spotlight: Some("x".into()),
                replay: true,
            },
            HelpVisitRecord {
                route: "/apps".into(),
                feature_highlight: "b".into(),
                spotlight: None,
                replay: false,
            },
        ];
        let merged = merge_local_into_server(&server, &local);
        assert_eq!(merged.len(), 2);
        assert!(!merged[0].replay);
        assert_eq!(merged[1].feature_highlight, "b");
    }
}
