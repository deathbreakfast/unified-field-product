//! Who sent each notification, looked up when the page loads.
//!
//! Products don't put the actor's name in notification text, because stored text
//! outlives account erasure. The row's `sender` reference is cleared when that
//! user is deleted (`SetNull`), so the inbox and bell read the sender's current
//! profile display name on every page load instead. A row with no sender (System
//! notices, or an erased account) shows [`UNKNOWN_SENDER`].
//!
//! Lookups run under the recipient's session Valence: the notification read
//! policy only returns the recipient's own rows, and Lepton's
//! `user_profile.display_name` is readable by any signed-in user.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::server::{NotificationDto, NotificationReadFilter, Page};

/// Label shown when a notification has no sender or the sender has no display name.
pub const UNKNOWN_SENDER: &str = "Someone";

/// A notification plus its sender's current display name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationRow {
    /// The notification as the API returns it.
    pub notification: NotificationDto,
    /// Sender's current profile display name, or [`UNKNOWN_SENDER`].
    pub sender_name: String,
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
            let sender_name =
                match notification_sender(&valence, notification.notification_id).await {
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

    /// The row's `sender`, or `None` when it has none or the row can't be read.
    async fn notification_sender(valence: &Valence, notification_id: Uuid) -> Option<RecordId> {
        let id = notification_id.to_string();
        match <NotificationModel as Model>::get(&id, valence, valence::use_!(r"In **notifications**, we **load your notification** to find who sent it, so the inbox and bell can show that person's current name. Only the recipient can load the row.")).await {
            Ok(row) => row.and_then(|n| n.sender().cloned()),
            Err(err) => {
                leptos::logging::warn!("notification sender lookup failed for {id}: {err}");
                None
            }
        }
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
