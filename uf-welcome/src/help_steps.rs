//! Seeded Help spotlight steps for the Welcome landing.

use leptos::prelude::*;
use uf_help_macros::help_spotlight_step;

/// Help step: featured apps card on welcome.
#[help_spotlight_step(
    route = "/welcome",
    feature_highlight = "welcome-featured",
    title = "Featured apps",
    spotlight = "welcome-featured-card",
    position = "bottom",
    order = 10
)]
#[component]
pub fn WelcomeFeaturedHelp() -> impl IntoView {
    view! {
        <p data-testid="help-step-welcome-featured">
            "Featured apps are curated shortcuts for promotional items."
        </p>
    }
}

/// Help step: View all apps on the featured card.
#[help_spotlight_step(
    route = "/welcome",
    feature_highlight = "welcome-featured-view-all",
    title = "View all apps",
    spotlight = "welcome-featured-view-all",
    position = "bottom",
    order = 20
)]
#[component]
pub fn WelcomeFeaturedViewAllHelp() -> impl IntoView {
    view! {
        <p data-testid="help-step-welcome-featured-view-all">
            "To see every application and product offering, click here."
        </p>
    }
}

/// Help step: recent apps card.
#[help_spotlight_step(
    route = "/welcome",
    feature_highlight = "welcome-recent",
    title = "Recent apps",
    spotlight = "welcome-recent-apps-card",
    position = "bottom",
    order = 30
)]
#[component]
pub fn WelcomeRecentHelp() -> impl IntoView {
    view! {
        <p data-testid="help-step-welcome-recent">
            "These are the apps you used most recently."
        </p>
    }
}

/// Help step: most-used apps card.
#[help_spotlight_step(
    route = "/welcome",
    feature_highlight = "welcome-most-used",
    title = "Most used",
    spotlight = "welcome-most-used-card",
    position = "bottom",
    order = 40
)]
#[component]
pub fn WelcomeMostUsedHelp() -> impl IntoView {
    view! {
        <p data-testid="help-step-welcome-most-used">
            "These are the apps you open most often."
        </p>
    }
}

/// Help step: popular apps card.
#[help_spotlight_step(
    route = "/welcome",
    feature_highlight = "welcome-popular",
    title = "Popular apps",
    spotlight = "welcome-popular-apps-card",
    position = "bottom",
    order = 50
)]
#[component]
pub fn WelcomePopularHelp() -> impl IntoView {
    view! {
        <p data-testid="help-step-welcome-popular">
            "These are popular apps across the product."
        </p>
    }
}

/// Ensure welcome help inventory is linked into the host binary.
pub fn ensure_help_steps_linked() {}
