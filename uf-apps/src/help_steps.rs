//! Seeded Help spotlight steps for the Apps directory and app overview.

use leptos::prelude::*;
use uf_help_macros::help_spotlight_step;

/// Help step: search the apps catalog.
#[help_spotlight_step(
    route = "/apps",
    feature_highlight = "apps-search",
    title = "Search apps",
    spotlight = "apps-search-input",
    position = "bottom",
    order = 10
)]
#[component]
pub fn AppsSearchHelp() -> impl IntoView {
    view! {
        <p data-testid="help-step-apps-search">
            "Use search to find installed apps by name."
        </p>
    }
}

/// Help step: application cards grid on the apps index.
#[help_spotlight_step(
    route = "/apps",
    feature_highlight = "apps-application-cards",
    title = "Application cards",
    spotlight = "apps-first-application-card",
    position = "bottom",
    order = 20
)]
#[component]
pub fn AppsApplicationCardsHelp() -> impl IntoView {
    view! {
        <p data-testid="help-step-apps-application-cards">
            "App results show up as cards here. Open one for more detail."
        </p>
    }
}

/// Help step: overview description on an app detail page.
#[help_spotlight_step(
    route = "/apps/:app_name",
    feature_highlight = "app-overview-more-information",
    title = "More information",
    spotlight = "app-overview-more-information",
    position = "bottom",
    order = 10
)]
#[component]
pub fn AppOverviewMoreInformationHelp() -> impl IntoView {
    view! {
        <p data-testid="help-step-app-overview-more-information">
            "The application description is here."
        </p>
    }
}

/// Help step: source repository link on an app detail page.
#[help_spotlight_step(
    route = "/apps/:app_name",
    feature_highlight = "app-overview-source-code",
    title = "Source code",
    spotlight = "app-overview-source-code",
    position = "bottom",
    order = 20
)]
#[component]
pub fn AppOverviewSourceCodeHelp() -> impl IntoView {
    view! {
        <p data-testid="help-step-app-overview-source-code">
            "The source repository is linked here when one is available."
        </p>
    }
}

/// Help step: documentation link on an app detail page.
#[help_spotlight_step(
    route = "/apps/:app_name",
    feature_highlight = "app-overview-documentation",
    title = "Documentation",
    spotlight = "app-overview-documentation",
    position = "bottom",
    order = 30
)]
#[component]
pub fn AppOverviewDocumentationHelp() -> impl IntoView {
    view! {
        <p data-testid="help-step-app-overview-documentation">
            "Docs for this app are linked here when published."
        </p>
    }
}

/// Help step: product link CTA on an app detail page.
#[help_spotlight_step(
    route = "/apps/:app_name",
    feature_highlight = "app-overview-product-link",
    title = "Product link",
    spotlight = "app-overview-product-link",
    position = "top",
    order = 40
)]
#[component]
pub fn AppOverviewProductLinkHelp() -> impl IntoView {
    view! {
        <p data-testid="help-step-app-overview-product-link">
            "Open the product from this link."
        </p>
    }
}

/// Ensure apps help inventory is linked into the host binary.
///
/// Empty function body; references every `#[help_spotlight_step]` in this module so
/// `inventory` submissions are retained. See [`uf_help`] crate docs for the full ladder.
pub fn ensure_help_steps_linked() {}
