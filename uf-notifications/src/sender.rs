//! Who sent each notification, looked up when the page loads.
//!
//! Products don't put the actor's name in notification text, because stored text
//! outlives account erasure. The row's `sender` reference is cleared when that
//! user is deleted (`SetNull`), so the inbox and bell read the sender's current
//! profile display name on every page load instead. A row with no sender (System
//! notices, or an erased account) shows [`UNKNOWN_SENDER`].
//!
//! The same row read gives each notification its absolute `created_at` (the DTO
//! only carries a relative label), and the sending app's display name comes from
//! the host's app registry.
//!
//! Lookups run under the recipient's session Valence: the notification read
//! policy only returns the recipient's own rows, and Lepton's
//! `user_profile.display_name` is readable by any signed-in user.

use chrono::{DateTime, Utc};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::server::{NotificationDto, NotificationInboxFilter, NotificationReadFilter, Page};

/// Label shown when a notification has no sender or the sender has no display name.
pub const UNKNOWN_SENDER: &str = "Someone";

/// A notification plus its sender's current display name, when it arrived and
/// which app sent it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationRow {
    /// The notification as the API returns it.
    pub notification: NotificationDto,
    /// Sender's current profile display name, or [`UNKNOWN_SENDER`].
    pub sender_name: String,
    /// When the notification was created; `None` when the row couldn't be read,
    /// in which case the view falls back to the DTO's relative label.
    pub created_at: Option<DateTime<Utc>>,
    /// The sending app's display name; `None` for an unregistered kind or app.
    pub app_name: Option<String>,
}

/// Counts shown on the inbox tabs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct InboxCounts {
    /// Every notification.
    pub all: usize,
    /// Notifications not yet read.
    pub unread: usize,
    /// Notifications whose kind asks the reader to act.
    pub waiting: usize,
}

/// An app that has registered notification kinds, for the inbox app filter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InboxApp {
    /// App id, as in [`NotificationDto::app_id`].
    pub id: String,
    /// The app's display name, or its id when the host doesn't register it.
    pub name: String,
}

/// The name to show for a sender: its display name, or [`UNKNOWN_SENDER`] when absent or blank.
#[must_use]
pub fn sender_label(display_name: Option<&str>) -> String {
    display_name
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map_or_else(|| UNKNOWN_SENDER.to_string(), str::to_string)
}

/// Inbox page of notifications with sender names.
///
/// Same arguments and auth as [`crate::server::get_notifications_page`].
///
/// # Errors
///
/// Returns the underlying page's [`ServerFnError`] (signed out, store failure) or a
/// session Valence error. A failed sender lookup falls back to [`UNKNOWN_SENDER`]
/// instead of failing the page.
#[server]
pub async fn get_notification_rows_page(
    /// Zero-based index of the first notification to return.
    offset: u32,
    /// Maximum number of notifications to return.
    limit: u32,
    /// Optional free-text search matched against title and message.
    query: Option<String>,
    /// Read-status filter to apply.
    read_filter: NotificationReadFilter,
) -> Result<Page<NotificationRow>, ServerFnError> {
    let page =
        uf_notifications_api::get_notifications_page(offset, limit, query, read_filter).await?;
    ssr::with_sender_names(page).await
}

/// Inbox page narrowed by tab, app and search, with sender names.
///
/// Same rows as [`crate::server::get_inbox_by_request`]: `filter.asks_action`
/// keeps kinds that ask the reader to act and `filter.app_id` keeps one app.
///
/// # Errors
///
/// Same as [`get_notification_rows_page`].
#[server]
pub async fn get_inbox_rows_page(
    /// Offset, limit and cursor; sort, filter and quick search are ignored.
    request: crate::server::PageRequest,
    /// Optional free-text search matched against title and message.
    query: Option<String>,
    /// Read-status filter to apply.
    read_filter: NotificationReadFilter,
    /// Asks-for-action and app filter.
    filter: NotificationInboxFilter,
) -> Result<Page<NotificationRow>, ServerFnError> {
    let page =
        uf_notifications_api::get_inbox_by_request(request, query, read_filter, filter).await?;
    ssr::with_sender_names(page).await
}

/// Counts for the inbox tabs: all, unread and waiting on you.
///
/// Counts cover the whole inbox, not the current search or app filter.
///
/// # Errors
///
/// Returns the first count's [`ServerFnError`] (signed out, store failure).
#[server]
pub async fn get_inbox_counts() -> Result<InboxCounts, ServerFnError> {
    let all = uf_notifications_api::get_notification_count().await?;
    let unread = uf_notifications_api::get_unread_count().await?;
    let waiting_page = uf_notifications_api::get_inbox_by_request(
        crate::server::PageRequest {
            offset: 0,
            limit: 1,
            ..Default::default()
        },
        None,
        NotificationReadFilter::All,
        NotificationInboxFilter {
            asks_action: true,
            app_id: None,
        },
    )
    .await?;
    let waiting = waiting_page
        .total_count
        .map_or(waiting_page.items.len(), |n| {
            usize::try_from(n).unwrap_or(usize::MAX)
        });
    Ok(InboxCounts {
        all,
        unread,
        waiting,
    })
}

/// Apps that registered notification kinds, sorted by name, for the app filter.
///
/// # Errors
///
/// Returns [`ServerFnError`] when the caller is not signed in.
#[server]
pub async fn get_inbox_apps() -> Result<Vec<InboxApp>, ServerFnError> {
    let kinds = uf_notifications_api::list_notification_kinds().await?;
    let mut apps: Vec<InboxApp> = Vec::new();
    for kind in kinds {
        if apps.iter().any(|a| a.id == kind.app) {
            continue;
        }
        let name = ssr::app_name(&kind.app).unwrap_or_else(|| kind.app.clone());
        apps.push(InboxApp { id: kind.app, name });
    }
    apps.sort_by_key(|a| a.name.to_lowercase());
    Ok(apps)
}

/// Bell page of unread notifications with sender names.
///
/// Same arguments and auth as [`crate::server::get_unread_notifications_page`].
///
/// # Errors
///
/// Same as [`get_notification_rows_page`].
#[server]
pub async fn get_unread_notification_rows_page(
    /// Zero-based index of the first notification to return.
    offset: u32,
    /// Maximum number of notifications to return.
    limit: u32,
) -> Result<Page<NotificationRow>, ServerFnError> {
    let page = uf_notifications_api::get_unread_notifications_page(offset, limit).await?;
    ssr::with_sender_names(page).await
}

#[cfg(feature = "ssr")]
mod ssr {
    use std::collections::HashMap;

    use chrono::{DateTime, Utc};
    use lepton_identity::generated::UserProfile;
    use leptos::prelude::ServerFnError;
    use uf_notifications_api::NotificationModel;
    use uuid::Uuid;
    use valence::{Model, RecordId, RecordPredicate, Valence};

    use super::{sender_label, NotificationRow, UNKNOWN_SENDER};
    use crate::server::{NotificationDto, Page};

    pub(super) async fn with_sender_names(
        page: Page<NotificationDto>,
    ) -> Result<Page<NotificationRow>, ServerFnError> {
        let ctx = uf_product::ssr::higgs().await?;
        let valence = uf_product::ssr::valence(&ctx)?;

        let mut names: HashMap<String, String> = HashMap::new();
        let mut items = Vec::with_capacity(page.items.len());
        for notification in page.items {
            let (sender, created_at) =
                notification_sender(&valence, notification.notification_id).await;
            let app_name = notification.app_id.as_deref().and_then(app_name);
            let sender_name = match sender {
                None => UNKNOWN_SENDER.to_string(),
                Some(sender) => {
                    let key = sender.to_string();
                    if let Some(name) = names.get(&key) {
                        name.clone()
                    } else {
                        let name = sender_display_name(&valence, sender).await;
                        names.insert(key, name.clone());
                        name
                    }
                }
            };
            items.push(NotificationRow {
                notification,
                sender_name,
                created_at,
                app_name,
            });
        }

        Ok(Page {
            items,
            has_more: page.has_more,
            total_count: page.total_count,
            next_request_offset: page.next_request_offset,
            next_cursor: page.next_cursor,
        })
    }

    /// The row's `sender` and `created_at`; `(None, None)` when the row can't be read.
    async fn notification_sender(
        valence: &Valence,
        notification_id: Uuid,
    ) -> (Option<RecordId>, Option<DateTime<Utc>>) {
        let id = notification_id.to_string();
        match <NotificationModel as Model>::get(&id, valence, valence::use_!(r"In **notifications**, we **load your notification** to find who sent it and when it arrived, so the inbox and bell can show that person's current name and the time. Only the recipient can load the row.")).await {
            Ok(Some(n)) => (n.sender().cloned(), Some(*n.created_at())),
            Ok(None) => (None, None),
            Err(err) => {
                leptos::logging::warn!("notification sender lookup failed for {id}: {err}");
                (None, None)
            }
        }
    }

    /// The display name the host registers for `app_id`.
    pub(super) fn app_name(app_id: &str) -> Option<String> {
        uf_product::AppRegistry::auto_discover()
            .get(app_id)
            .map(|reg| reg.name.to_string())
    }

    /// The sender's current profile display name, or [`UNKNOWN_SENDER`].
    async fn sender_display_name(valence: &Valence, sender: RecordId) -> String {
        let profile = UserProfile::query(valence, valence::use_!(r"In **notifications**, we **read the sender's public display name** from their profile when you open the inbox or bell, so you see who acted without the name ever being stored in the notification."))
            .where_user(RecordPredicate::Equals(sender.clone()))
            .first()
            .await;
        match profile {
            Ok(profile) => sender_label(profile.as_ref().map(|p| p.display_name().as_str())),
            Err(err) => {
                leptos::logging::warn!("sender profile lookup failed for {sender}: {err}");
                UNKNOWN_SENDER.to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sender_label_uses_display_name_happy() {
        assert_eq!(sender_label(Some("Ada Lovelace")), "Ada Lovelace");
        assert_eq!(sender_label(Some("  Ada  ")), "Ada");
    }

    #[test]
    fn sender_label_falls_back_to_someone_sad() {
        assert_eq!(sender_label(None), UNKNOWN_SENDER);
        assert_eq!(sender_label(Some("")), UNKNOWN_SENDER);
        assert_eq!(sender_label(Some("   ")), UNKNOWN_SENDER);
    }
}
