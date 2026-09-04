//! Anonymous localStorage mirror for help tour visits.

use super::visits::HelpVisitRecord;

/// Browser storage key for anon help tour progress.
pub const LOCAL_STORAGE_KEY: &str = "uf.help.tour_steps";

#[cfg(target_arch = "wasm32")]
pub fn read_local_visits() -> Vec<HelpVisitRecord> {
    use leptos::web_sys;
    let Some(window) = web_sys::window() else {
        return Vec::new();
    };
    let Ok(Some(storage)) = window.local_storage() else {
        return Vec::new();
    };
    let Ok(Some(raw)) = storage.get_item(LOCAL_STORAGE_KEY) else {
        return Vec::new();
    };
    serde_json::from_str(&raw).unwrap_or_default()
}

#[cfg(not(target_arch = "wasm32"))]
#[must_use]
pub fn read_local_visits() -> Vec<HelpVisitRecord> {
    Vec::new()
}

#[cfg(target_arch = "wasm32")]
pub fn write_local_visits(visits: &[HelpVisitRecord]) {
    use leptos::web_sys;
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(json) = serde_json::to_string(visits) {
                let _ = storage.set_item(LOCAL_STORAGE_KEY, &json);
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn write_local_visits(_visits: &[HelpVisitRecord]) {}

/// Visits for a single pathname from local storage.
///
/// Rows may store an inventory pattern (for example `"/apps/:app_name"`) while
/// `pathname` is the live browser path (`"/apps/welcome"`).
#[must_use]
pub fn read_local_visits_for_route(pathname: &str) -> Vec<HelpVisitRecord> {
    read_local_visits()
        .into_iter()
        .filter(|v| crate::route_matches(&v.route, pathname))
        .collect()
}

/// Upsert mark-seen rows into local storage (`replay = false`).
pub fn local_mark_steps_seen(steps: &[super::visits::HelpStepKey]) {
    let mut visits = read_local_visits();
    for step in steps {
        if let Some(existing) = visits
            .iter_mut()
            .find(|v| v.route == step.route && v.feature_highlight == step.feature_highlight)
        {
            existing.replay = false;
            if step.spotlight.is_some() {
                existing.spotlight = step.spotlight.clone();
            }
        } else {
            visits.push(HelpVisitRecord {
                route: step.route.clone(),
                feature_highlight: step.feature_highlight.clone(),
                spotlight: step.spotlight.clone(),
                replay: false,
            });
        }
    }
    write_local_visits(&visits);
}

/// Set `replay = true` for all local visits matching `pathname`.
pub fn local_request_replay_for_route(pathname: &str) {
    let visits = read_local_visits();
    let next = super::visits::apply_replay_for_route(&visits, pathname);
    // If there are no rows yet, invent placeholder rows from inventory so replay
    // still forces a tour of registered steps. Store the inventory route key
    // (pattern or exact), not the live slug pathname.
    let mut next = next;
    for step in crate::collect_help_steps_for_route(pathname) {
        if !next
            .iter()
            .any(|v| v.route == step.route && v.feature_highlight == step.feature_highlight)
        {
            next.push(HelpVisitRecord {
                route: step.route.to_string(),
                feature_highlight: step.feature_highlight.to_string(),
                spotlight: step.spotlight.map(str::to_string),
                replay: true,
            });
        }
    }
    write_local_visits(&next);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::visits::HelpStepKey;

    #[test]
    fn corrupt_local_json_defaults_empty_on_native() {
        // Native builds always return empty; wasm parse-fail path mirrors this.
        assert!(read_local_visits().is_empty());
    }

    #[test]
    fn local_mark_and_replay_roundtrip_native_noop() {
        local_mark_steps_seen(&[HelpStepKey {
            route: "/apps".into(),
            feature_highlight: "a".into(),
            spotlight: None,
        }]);
        local_request_replay_for_route("/apps");
        assert!(read_local_visits().is_empty());
    }
}
