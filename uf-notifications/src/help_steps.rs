//! Help spotlight steps for the notification bell and inbox.

use leptos::prelude::*;
use uf_help_macros::help_spotlight_step;

/// Help step: unread badge on the shell bell.
///
/// Bound to `/notifications` (product inbox shell), not `/`. On marketing
/// site hosts, `/` has no app-bar bell.
#[help_spotlight_step(
    route = "/notifications",
    feature_highlight = "notifications-bell",
    title = "Notification bell",
    spotlight = "notification-bell",
    position = "bottom",
    order = 10
)]
#[component]
pub fn NotificationsBellHelp() -> impl IntoView {
    view! {
        <p data-testid="help-step-notifications-bell">
            "The bell shows unread notifications for the signed-in user."
        </p>
    }
}

/// Help step: inbox page.
#[help_spotlight_step(
    route = "/notifications",
    feature_highlight = "notifications-inbox",
    title = "Inbox",
    spotlight = "notifications-inbox-page",
    position = "top",
    order = 20
)]
#[component]
pub fn NotificationsInboxHelp() -> impl IntoView {
    view! {
        <p data-testid="help-step-notifications-inbox">
            "Open Inbox to search, filter, and mark notifications read."
        </p>
    }
}

/// Help step: left-nav inbox link.
#[help_spotlight_step(
    route = "/notifications",
    feature_highlight = "notifications-nav",
    title = "Inbox nav",
    spotlight = "nav-notifications-inbox",
    position = "right",
    order = 30
)]
#[component]
pub fn NotificationsNavHelp() -> impl IntoView {
    view! {
        <p data-testid="help-step-notifications-nav">
            "Use Inbox in the left nav to return to this page."
        </p>
    }
}

/// Link the notifications help inventory into the host binary.
pub fn ensure_help_steps_linked() {}
