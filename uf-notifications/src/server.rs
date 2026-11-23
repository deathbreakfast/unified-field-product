//! Notification server functions
//!
//! Re-exports the notification API from `uf-notifications-api` so UI crates can
//! import list / mark-read / badge helpers from [`mod@crate::server`] without taking
//! a direct `uf-notifications-api` dependency in every module.

pub use uf_notifications_api::{
    get_notification_count, get_notifications_page, get_today_count, get_unread_count,
    get_unread_notifications_page, get_unread_notifications_preview, list_notifications,
    mark_all_notifications_read, mark_notification_read, mark_notification_unread,
    subscribe_get_unread_count, NotificationDto, NotificationReadFilter, Page,
};

pub use crate::sender::{
    get_notification_rows_page, get_unread_notification_rows_page, sender_label, NotificationRow,
    UNKNOWN_SENDER,
};

#[cfg(feature = "dev-tools")]
pub use uf_notifications_api::create_test_notification;
