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

/// Centered intro on `/help-fixture/deferred`; always presentable.
#[help_spotlight_step(
    route = "/help-fixture/deferred",
    feature_highlight = "deferred-intro",
    title = "Deferred anchors",
    order = 10
)]
#[component]
pub fn DeferredIntroHelp() -> impl IntoView {
    view! {
        <p data-testid="help-step-deferred-intro">
            "Steps on this page wait until their element is on screen."
        </p>
    }
}

/// Anchored below the fold; visible, so the spotlight scrolls to it.
#[help_spotlight_step(
    route = "/help-fixture/deferred",
    feature_highlight = "deferred-below-fold",
    title = "Below the fold",
    spotlight = "help-fixture-below-fold",
    order = 20
)]
#[component]
pub fn DeferredBelowFoldHelp() -> impl IntoView {
    view! {
        <p data-testid="help-step-deferred-below-fold">
            "This element starts under the fold."
        </p>
    }
}

/// Anchored on an element that only mounts after "Mount panel" is clicked.
#[help_spotlight_step(
    route = "/help-fixture/deferred",
    feature_highlight = "deferred-mounted",
    title = "Mounted later",
    spotlight = "help-fixture-mounted",
    order = 30
)]
#[component]
pub fn DeferredMountedHelp() -> impl IntoView {
    view! {
        <p data-testid="help-step-deferred-mounted">
            "This panel mounted after the page loaded."
        </p>
    }
}

/// Anchored on an element that stays `display: none` until "Show hidden" is clicked.
#[help_spotlight_step(
    route = "/help-fixture/deferred",
    feature_highlight = "deferred-css-hidden",
    title = "Hidden with CSS",
    spotlight = "help-fixture-css-hidden",
    order = 40
)]
#[component]
pub fn DeferredCssHiddenHelp() -> impl IntoView {
    view! {
        <p data-testid="help-step-deferred-css-hidden">
            "This panel was in the page but hidden."
        </p>
    }
}

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
