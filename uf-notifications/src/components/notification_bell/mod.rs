//! Notification bell dropdown for product app bars.
//!
//! Hosts mount [`NotificationBell`] via inventory
//! (`uf_integrations::ShellNotificationBellContribution`) when
//! `offering-notifications` is enabled, or override with
//! [`uf_integrations::provide_shell_notification_bell`].

mod item;
mod menu;
mod root;
mod trigger;

pub(super) use crate::safe_url::NOTIFICATIONS_PATH;

pub use root::NotificationBell;
