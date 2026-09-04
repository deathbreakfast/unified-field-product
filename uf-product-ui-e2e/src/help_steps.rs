//! Help spotlight seeds for the product UI e2e host (unguarded routes).

use leptos::prelude::*;
use uf_help_macros::help_spotlight_step;

/// Tour step on `/coming-soon` so anon scenarios are not blocked by Apps auth.
///
/// Omits `spotlight` so Orbital centers the panel in the viewport (no cutout).
#[help_spotlight_step(
    route = "/coming-soon",
    feature_highlight = "coming-soon-intro",
    title = "Coming Soon Intro",
    order = 10
)]
#[component]
pub fn ComingSoonHelp() -> impl IntoView {
    view! {
        <p data-testid="help-step-coming-soon">
            "This page stands in for features that are not ready yet."
        </p>
    }
}

/// Force-link e2e help inventory.
pub fn ensure_help_steps_linked() {}

/// Tour step on `/gate/permission` so we can assert Help stays off while the
/// permission-required modal is up.
#[help_spotlight_step(
    route = "/gate/permission",
    feature_highlight = "gate-permission-intro",
    title = "Permission gate",
    order = 10
)]
#[component]
pub fn GatePermissionHelp() -> impl IntoView {
    view! {
        <p data-testid="help-step-gate-permission">
            "This copy should not appear while Permission required is showing."
        </p>
    }
}
