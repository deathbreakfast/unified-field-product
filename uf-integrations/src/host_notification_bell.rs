//! Host-provided notification bell for product shells.
//!
//! Prefer [`provide_shell_notification_bell`] when the host wants an explicit
//! override. Otherwise, with `uf-integrations` feature `offering-notifications`
//! (included in `full`), link `uf-notifications` so its inventory contribution
//! fills [`HostNotificationBell`].

use std::sync::Arc;

use leptos::prelude::*;

/// Factory that renders the host's notification bell (`uf_notifications::NotificationBell`).
#[derive(Clone)]
pub struct ShellNotificationBellFactory {
    render: Arc<dyn Fn() -> AnyView + Send + Sync>,
}

impl ShellNotificationBellFactory {
    /// Create a factory from a view builder.
    pub fn new<F, V>(f: F) -> Self
    where
        F: Fn() -> V + Send + Sync + 'static,
        V: IntoView,
    {
        Self {
            render: Arc::new(move || f().into_any()),
        }
    }

    /// Render the host notification bell.
    pub fn render(&self) -> AnyView {
        (self.render)()
    }
}

/// Provide the notification bell for product layouts that call [`HostNotificationBell`].
///
/// Host override wins over any [`ShellNotificationBellContribution`] inventory row.
pub fn provide_shell_notification_bell<F, V>(f: F)
where
    F: Fn() -> V + Send + Sync + 'static,
    V: IntoView,
{
    provide_context(ShellNotificationBellFactory::new(f));
}

/// Inventory contribution for the default shell notification bell.
///
/// `uf-notifications` submits its `NotificationBell` here. Hosts that enable
/// `offering-notifications` / `full` should also depend on `uf-notifications` so
/// the inventory row is linked (same pattern as `offering-apps`).
pub struct ShellNotificationBellContribution {
    /// Render the bell as an [`AnyView`].
    pub render: fn() -> AnyView,
}

impl ShellNotificationBellContribution {
    /// Construct a contribution for inventory registration.
    pub const fn new(render: fn() -> AnyView) -> Self {
        Self { render }
    }
}

inventory::collect!(ShellNotificationBellContribution);

/// No-op touch point so offering crates can force-link inventory into the binary.
pub fn register_shell_notification_bell() {}

/// First inventory contribution, if any offering crate submitted one.
pub fn collect_shell_notification_bell() -> Option<&'static ShellNotificationBellContribution> {
    inventory::iter::<ShellNotificationBellContribution>
        .into_iter()
        .next()
}

/// Renders the host-provided notification bell, inventory fallback, or nothing.
#[component]
pub fn HostNotificationBell() -> impl IntoView {
    if let Some(factory) = use_context::<ShellNotificationBellFactory>() {
        return factory.render();
    }
    #[cfg(feature = "offering-notifications")]
    {
        if let Some(contrib) = collect_shell_notification_bell() {
            return (contrib.render)();
        }
    }
    ().into_any()
}
