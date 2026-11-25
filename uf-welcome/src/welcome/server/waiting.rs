//! Waiting on you: unread notifications that ask the viewer to act.

use leptos::prelude::*;
use uf_notifications_api::NotificationDto;

/// Most items the Waiting on you card shows.
pub const WAITING_LIMIT: u32 = 5;

/// `user`'s unread notifications whose kind asks them to act, newest first,
/// at most [`WAITING_LIMIT`]. Reading a notification removes it.
///
/// Runs as the actor on `valence`; the notification store only returns the
/// actor's own rows.
///
/// # Errors
///
/// Returns [`uf_notifications_api::NotificationOpsError`] when the inbox
/// query fails.
#[cfg(feature = "ssr")]
pub async fn waiting_for_user(
    valence: &valence::Valence,
    user: valence::RecordId,
) -> Result<Vec<NotificationDto>, uf_notifications_api::NotificationOpsError> {
    use uf_notifications_api::{
        inbox_page_for_request, NotificationInboxFilter, NotificationReadFilter, PageRequest,
    };

    let page = inbox_page_for_request(
        valence,
        user,
        &PageRequest::new(0, WAITING_LIMIT),
        None,
        NotificationReadFilter::Unread,
        &NotificationInboxFilter {
            asks_action: true,
            app_id: None,
        },
    )
    .await?;
    Ok(page.items)
}

/// The viewer's unread notifications that ask them to act, newest first, at
/// most 5.
///
/// Returns an empty list when nobody is signed in.
///
/// # Errors
///
/// Returns [`ServerFnError`] when the request context or session Valence
/// cannot be built, or the inbox query fails.
#[uf_product_macros::server]
pub async fn get_waiting_on_me() -> Result<Vec<NotificationDto>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let Some((valence, user)) = super::owner_session().await? else {
            return Ok(Vec::new());
        };
        waiting_for_user(&valence, user)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(Vec::new())
    }
}
