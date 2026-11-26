//! Pinned apps: the apps a person keeps at the top of their Welcome page.
//!
//! Rows are private to their owner ([`crate::generated::WelcomePinnedApp`],
//! owner-only policies, deleted with the account). Every call runs as the
//! session actor; nobody else, WelcomeAdmin included, can read them.

use leptos::prelude::*;

use super::featured::AppLinkDto;

#[cfg(feature = "ssr")]
pub use store::{list_pinned, pin, reorder_pinned, unpin, PinnedError};

#[cfg(feature = "ssr")]
mod store {
    use std::fmt;

    use chrono::Utc;
    use valence::{Model, RecordId, RecordPredicate, StringPredicate, Valence};

    use crate::generated::WelcomePinnedApp;

    /// Failure reading or changing pinned apps.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum PinnedError {
        /// `app_id` is not registered on this host.
        UnknownApp(String),
        /// `app_id` is already pinned.
        Duplicate(String),
        /// `app_id` is not pinned.
        NotFound(String),
        /// Valence failure.
        Service {
            /// Operation label (`list`, `pin`, `unpin`, `reorder`).
            operation: &'static str,
            /// Backend message.
            message: String,
        },
    }

    impl PinnedError {
        fn service(operation: &'static str, e: impl fmt::Display) -> Self {
            Self::Service {
                operation,
                message: e.to_string(),
            }
        }
    }

    impl fmt::Display for PinnedError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::UnknownApp(app_id) => write!(f, "unknown app_id: {app_id}"),
                Self::Duplicate(app_id) => write!(f, "app already pinned: {app_id}"),
                Self::NotFound(app_id) => write!(f, "app not pinned: {app_id}"),
                Self::Service { operation, message } => {
                    write!(f, "pinned apps {operation} failed: {message}")
                }
            }
        }
    }

    impl std::error::Error for PinnedError {}

    async fn rows(v: &Valence, user: &RecordId) -> Result<Vec<WelcomePinnedApp>, PinnedError> {
        WelcomePinnedApp::query(v, valence::use_!(r"On **Welcome**, we **load the apps you pinned** and their order, and **show them to you** in your **Pinned** list. Only your own pins are read; nobody else, admins included, can see them."))
            .where_user(RecordPredicate::Equals(user.clone()))
            .await
            .map_err(|e| PinnedError::service("list", e))
    }

    async fn find(
        v: &Valence,
        user: &RecordId,
        app_id: &str,
    ) -> Result<Option<WelcomePinnedApp>, PinnedError> {
        WelcomePinnedApp::query(v, valence::use_!(r"When you **pin, unpin or reorder an app** on Welcome, we **look up your pin for that app** to check whether it exists before changing it. Only your own pins are read."))
            .where_user(RecordPredicate::Equals(user.clone()))
            .where_app_id(StringPredicate::Equals(app_id.to_string()))
            .first()
            .await
            .map_err(|e| PinnedError::service("find", e))
    }

    /// App ids `user` pinned, in their order. Apps this host no longer
    /// registers are left out.
    ///
    /// # Errors
    ///
    /// Returns [`PinnedError::Service`] when the Valence query fails.
    pub async fn list_pinned(v: &Valence, user: &RecordId) -> Result<Vec<String>, PinnedError> {
        let mut pins: Vec<(i64, String)> = rows(v, user)
            .await?
            .into_iter()
            .map(|row| (*row.ordinal(), row.app_id().clone()))
            .collect();
        pins.sort();
        Ok(pins
            .into_iter()
            .map(|(_, app_id)| app_id)
            .filter(|app_id| super::super::featured::app_link_for_id(app_id).is_some())
            .collect())
    }

    /// Pin `app_id` for `user`, after their existing pins.
    ///
    /// # Errors
    ///
    /// Returns [`PinnedError::UnknownApp`] for an unregistered app,
    /// [`PinnedError::Duplicate`] when it is already pinned, or
    /// [`PinnedError::Service`] when Valence fails.
    pub async fn pin(v: &Valence, user: &RecordId, app_id: &str) -> Result<(), PinnedError> {
        if super::super::featured::app_link_for_id(app_id).is_none() {
            return Err(PinnedError::UnknownApp(app_id.to_string()));
        }
        let existing = rows(v, user).await?;
        if existing.iter().any(|row| row.app_id() == app_id) {
            return Err(PinnedError::Duplicate(app_id.to_string()));
        }
        let next = existing
            .iter()
            .map(|row| *row.ordinal())
            .max()
            .map_or(0, |max| max + 1);
        let row = WelcomePinnedApp::new(user.clone(), app_id.to_string(), next, Utc::now())
            .map_err(|e| PinnedError::service("pin", e))?;
        WelcomePinnedApp::create(row, v, valence::use_!(r"When you **pin an app** on Welcome, we **save which app you pinned and where it goes in your list**, so it shows in your **Pinned** list next time. Only you can read it, and it is deleted when you unpin the app or your account is deleted."))
            .await
            .map_err(|e| PinnedError::service("pin", e))?;
        Ok(())
    }

    /// Unpin `app_id` for `user`.
    ///
    /// # Errors
    ///
    /// Returns [`PinnedError::NotFound`] when the app is not pinned, or
    /// [`PinnedError::Service`] when Valence fails.
    pub async fn unpin(v: &Valence, user: &RecordId, app_id: &str) -> Result<(), PinnedError> {
        // Found through the owner-only query, so the row is the caller's own.
        let row = find(v, user, app_id)
            .await?
            .ok_or_else(|| PinnedError::NotFound(app_id.to_string()))?;
        let id = row
            .id()
            .map(|r| r.id().to_string())
            .filter(|id| !id.is_empty())
            .ok_or_else(|| PinnedError::service("unpin", "pinned row has no record id"))?;
        // Delete now through the backend instead of `Model::delete`: that only
        // marks the row `pending_deletion`, which would block re-pinning the
        // same app until a deletion worker runs. The row came from the
        // owner-only query above, so skipping privacy here is safe.
        let table = WelcomePinnedApp::table_name();
        let backend = v
            .backend_for_table(table)
            .map_err(|e| PinnedError::service("unpin", e))?;
        valence::delete_record(backend.as_ref(), table, &id, valence::use_!(r"When you **unpin an app** on Welcome, we **delete your pin for that app** so it leaves your **Pinned** list."))
            .await
            .map_err(|e| PinnedError::service("unpin", e))?;
        valence::read_cache::invalidate(table, &id);
        Ok(())
    }

    /// Set the order of `user`'s pins to `app_ids` (first is shown first).
    ///
    /// # Errors
    ///
    /// Returns [`PinnedError::NotFound`] when an app in `app_ids` is not
    /// pinned, or [`PinnedError::Service`] when Valence fails.
    pub async fn reorder_pinned(
        v: &Valence,
        user: &RecordId,
        app_ids: &[String],
    ) -> Result<(), PinnedError> {
        let now = Utc::now();
        for (ordinal, app_id) in (0_i64..).zip(app_ids) {
            let row = find(v, user, app_id)
                .await?
                .ok_or_else(|| PinnedError::NotFound(app_id.clone()))?;
            row.get_mutable(v, valence::use_!(r"When you **reorder your pinned apps** on Welcome, we **save each pin's new position** so your **Pinned** list keeps the order you chose."))
                .set_ordinal(ordinal)
                .map_err(|e| PinnedError::service("reorder", e))?
                .set_updated_at(now)
                .map_err(|e| PinnedError::service("reorder", e))?
                .commit()
                .await
                .map_err(|e| PinnedError::service("reorder", e))?;
        }
        Ok(())
    }
}

#[cfg(feature = "ssr")]
fn map_pinned_error(e: PinnedError) -> ServerFnError {
    ServerFnError::new(e.to_string())
}

/// The viewer's pinned apps, in their order.
///
/// Returns an empty list when nobody is signed in.
///
/// # Errors
///
/// Returns [`ServerFnError`] when the request context or session Valence
/// cannot be built, or the query fails.
#[uf_product_macros::server]
pub async fn get_pinned_apps() -> Result<Vec<AppLinkDto>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let Some((valence, user)) = super::owner_session().await? else {
            return Ok(Vec::new());
        };
        let pins = list_pinned(&valence, &user)
            .await
            .map_err(map_pinned_error)?;
        Ok(pins
            .iter()
            .filter_map(|app_id| super::featured::app_link_for_id(app_id))
            .collect())
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(Vec::new())
    }
}

/// Pin `app_id` to the end of the viewer's pinned apps.
///
/// # Errors
///
/// Returns [`ServerFnError`] when nobody is signed in, the app is unknown or
/// already pinned, or Valence fails.
#[uf_product_macros::server]
pub async fn pin_app(app_id: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let (valence, user) = super::require_owner_session().await?;
        pin(&valence, &user, &app_id)
            .await
            .map_err(map_pinned_error)
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = app_id;
        Err(ServerFnError::new("ssr only"))
    }
}

/// Unpin `app_id` from the viewer's pinned apps.
///
/// # Errors
///
/// Returns [`ServerFnError`] when nobody is signed in, the app is not pinned,
/// or Valence fails.
#[uf_product_macros::server]
pub async fn unpin_app(app_id: String) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let (valence, user) = super::require_owner_session().await?;
        unpin(&valence, &user, &app_id)
            .await
            .map_err(map_pinned_error)
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = app_id;
        Err(ServerFnError::new("ssr only"))
    }
}

/// Reorder the viewer's pinned apps to `app_ids`.
///
/// # Errors
///
/// Returns [`ServerFnError`] when nobody is signed in, an app in `app_ids` is
/// not pinned, or Valence fails.
#[uf_product_macros::server]
pub async fn reorder_pinned_apps(app_ids: Vec<String>) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let (valence, user) = super::require_owner_session().await?;
        reorder_pinned(&valence, &user, &app_ids)
            .await
            .map_err(map_pinned_error)
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = app_ids;
        Err(ServerFnError::new("ssr only"))
    }
}
