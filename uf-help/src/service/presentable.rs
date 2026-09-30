//! Which pending steps can show right now, and when the player should open.
//!
//! A step with a `spotlight` id only shows once that element is on screen, so
//! steps for UI that renders later (a wizard stage, a loaded panel, a
//! role-gated control) wait instead of pointing at nothing. Waiting steps stay
//! pending: they are never marked seen until they have actually been shown.

use super::visits::HelpStepKey;
use crate::HelpStepDescriptor;

/// Spotlight id the step anchors to, or `None` for a viewport-centered step.
fn anchor_id(step: &HelpStepDescriptor) -> Option<&'static str> {
    step.spotlight.filter(|id| !id.is_empty())
}

/// Pending steps that can show now, in inventory order.
///
/// Centered steps always qualify. Anchored steps qualify only when
/// `anchor_visible` reports their element as on screen.
pub(crate) fn presentable_steps(
    pending: &[&'static HelpStepDescriptor],
    anchor_visible: impl Fn(&str) -> bool,
) -> Vec<&'static HelpStepDescriptor> {
    pending
        .iter()
        .copied()
        .filter(|step| anchor_id(step).is_none_or(&anchor_visible))
        .collect()
}

/// Visit keys for the steps a tour shows; only these get marked seen.
pub(crate) fn presented_keys(presented: &[&'static HelpStepDescriptor]) -> Vec<HelpStepKey> {
    presented
        .iter()
        .map(|step| HelpStepKey {
            route: step.route.to_string(),
            feature_highlight: step.feature_highlight.to_string(),
            spotlight: anchor_id(step).map(str::to_string),
        })
        .collect()
}

/// Whether the player should open a tour for `current`.
///
/// Requires the same non-empty set on two consecutive polls so a page that is
/// still loading does not open a tour with half its steps, and an anchor that
/// flickers in and out never opens one.
pub(crate) fn should_open(
    previous: &[HelpStepKey],
    current: &[HelpStepKey],
    client_ready: bool,
    gated: bool,
) -> bool {
    client_ready && !gated && !current.is_empty() && previous == current
}

#[cfg(test)]
mod tests {
    use super::*;
    use leptos::prelude::*;

    fn leak(step: HelpStepDescriptor) -> &'static HelpStepDescriptor {
        Box::leak(Box::new(step))
    }

    fn step(
        feature_highlight: &'static str,
        spotlight: Option<&'static str>,
    ) -> &'static HelpStepDescriptor {
        leak(HelpStepDescriptor {
            route: "/fixture",
            feature_highlight,
            title: feature_highlight,
            spotlight,
            order: 10,
            position: None,
            render: || ().into_any(),
        })
    }

    fn names(steps: &[&HelpStepDescriptor]) -> Vec<&'static str> {
        steps.iter().map(|s| s.feature_highlight).collect()
    }

    #[test]
    fn presentable_keeps_unanchored_and_visible() {
        let pending = [
            step("intro", None),
            step("table", Some("table")),
            step("form", Some("form")),
        ];
        let shown = presentable_steps(&pending, |id| id == "table" || id == "form");
        assert_eq!(names(&shown), ["intro", "table", "form"]);
    }

    #[test]
    fn presentable_drops_absent_anchor() {
        let pending = [
            step("intro", None),
            step("live-controls", Some("live-controls")),
            step("history", Some("history")),
        ];
        let shown = presentable_steps(&pending, |id| id == "history");
        assert_eq!(names(&shown), ["intro", "history"]);
    }

    #[test]
    fn empty_spotlight_is_unanchored() {
        let pending = [step("centered", Some(""))];
        let shown = presentable_steps(&pending, |_| false);
        assert_eq!(names(&shown), ["centered"]);
        assert_eq!(presented_keys(&shown)[0].spotlight, None);
    }

    #[test]
    fn presented_keys_excludes_waiting_steps() {
        let pending = [
            step("define", Some("define")),
            step("backtest", Some("backtest")),
        ];
        let shown = presentable_steps(&pending, |id| id == "define");
        let keys = presented_keys(&shown);
        assert_eq!(
            keys,
            [HelpStepKey {
                route: "/fixture".into(),
                feature_highlight: "define".into(),
                spotlight: Some("define".into()),
            }]
        );
    }

    #[test]
    fn should_open_requires_stable_nonempty_set() {
        let keys = presented_keys(&[step("intro", None)]);
        assert!(
            !should_open(&[], &keys, true, false),
            "first sighting waits a poll"
        );
        assert!(should_open(&keys, &keys, true, false));
    }

    #[test]
    fn should_open_false_when_empty_gated_or_not_ready() {
        let keys = presented_keys(&[step("intro", None)]);
        assert!(!should_open(&[], &[], true, false));
        assert!(
            !should_open(&keys, &keys, true, true),
            "access gate suppresses tours"
        );
        assert!(
            !should_open(&keys, &keys, false, false),
            "SSR / pre-hydrate never opens"
        );
    }

    #[test]
    fn should_open_false_when_set_changes_between_polls() {
        let intro = step("intro", None);
        let table = step("table", Some("table"));
        let loading = presented_keys(&[intro]);
        let loaded = presented_keys(&[intro, table]);
        assert!(!should_open(&loading, &loaded, true, false));
        assert!(
            !should_open(&loaded, &loading, true, false),
            "flicker back does not open"
        );
        assert!(should_open(&loaded, &loaded, true, false));
    }
}
