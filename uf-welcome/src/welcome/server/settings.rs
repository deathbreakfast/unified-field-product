//! Welcome settings: per-person choices for the Welcome page.
//!
//! One owner-only row per person ([`crate::generated::WelcomeSettings`],
//! deleted with the account). Nobody else, WelcomeAdmin included, can read it.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// The viewer's Welcome settings.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WelcomeSettingsDto {
    /// Whether the viewer dismissed the first-visit Start here card.
    pub start_here_hidden: bool,
}

#[cfg(feature = "ssr")]
pub use store::{load_settings, save_start_here_hidden, SettingsError};

#[cfg(feature = "ssr")]
mod store {
    use std::fmt;

    use chrono::Utc;
    use valence::{Model, RecordId, RecordPredicate, Valence};

    use super::WelcomeSettingsDto;
    use crate::generated::WelcomeSettings;

    /// Failure reading or saving Welcome settings.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SettingsError {
        /// Operation that failed (`load`, `save`).
        pub operation: &'static str,
        /// Backend message.
        pub message: String,
    }

    impl SettingsError {
        fn new(operation: &'static str, e: impl fmt::Display) -> Self {
            Self {
                operation,
                message: e.to_string(),
            }
        }
    }

    impl fmt::Display for SettingsError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "welcome settings {} failed: {}",
                self.operation, self.message
            )
        }
    }

    impl std::error::Error for SettingsError {}

    async fn find(v: &Valence, user: &RecordId) -> Result<Option<WelcomeSettings>, SettingsError> {
        WelcomeSettings::query(v, valence::use_!(r"On **Welcome**, we **load your Welcome settings**, such as whether you dismissed the **Start here** card, so the page shows or hides it for you. Only your own settings are read; nobody else, admins included, can see them."))
            .where_user(RecordPredicate::Equals(user.clone()))
            .first()
            .await
            .map_err(|e| SettingsError::new("load", e))
    }

    /// `user`'s settings, or the defaults when they have saved none.
    ///
    /// # Errors
    ///
    /// Returns [`SettingsError`] when the Valence query fails.
    pub async fn load_settings(
        v: &Valence,
        user: &RecordId,
    ) -> Result<WelcomeSettingsDto, SettingsError> {
        Ok(find(v, user)
            .await?
            .map(|row| WelcomeSettingsDto {
                start_here_hidden: *row.start_here_hidden(),
            })
            .unwrap_or_default())
    }

    /// Save whether `user` hid the Start here card.
    ///
    /// # Errors
    ///
    /// Returns [`SettingsError`] when the Valence query, create or update fails.
    pub async fn save_start_here_hidden(
        v: &Valence,
        user: &RecordId,
        hidden: bool,
    ) -> Result<(), SettingsError> {
        let now = Utc::now();
        if let Some(row) = find(v, user).await? {
            row.get_mutable(v, valence::use_!(r"When you **hide or bring back the Start here card** on Welcome, we **save that choice in your Welcome settings** so the page remembers it on your next visit."))
                .set_start_here_hidden(hidden)
                .map_err(|e| SettingsError::new("save", e))?
                .set_updated_at(now)
                .map_err(|e| SettingsError::new("save", e))?
                .commit()
                .await
                .map_err(|e| SettingsError::new("save", e))?;
            return Ok(());
        }
        let row = WelcomeSettings::new(user.clone(), hidden, now)
            .map_err(|e| SettingsError::new("save", e))?;
        WelcomeSettings::create(row, v, valence::use_!(r"The first time you **hide or bring back the Start here card** on Welcome, we **create your Welcome settings** to remember that choice. Only you can read them, and they are deleted with your account."))
            .await
            .map_err(|e| SettingsError::new("save", e))?;
        Ok(())
    }
}

/// The viewer's Welcome settings; defaults when nobody is signed in or none
/// are saved.
///
/// # Errors
///
/// Returns [`ServerFnError`] when the request context or session Valence
/// cannot be built, or the query fails.
#[uf_product_macros::server]
pub async fn get_welcome_settings() -> Result<WelcomeSettingsDto, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let Some((valence, user)) = super::owner_session().await? else {
            return Ok(WelcomeSettingsDto::default());
        };
        load_settings(&valence, &user)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))
    }
    #[cfg(not(feature = "ssr"))]
    {
        Ok(WelcomeSettingsDto::default())
    }
}

/// Hide (or bring back) the first-visit Start here card for the viewer.
///
/// # Errors
///
/// Returns [`ServerFnError`] when nobody is signed in, the session Valence
/// cannot be built, or the save fails.
#[uf_product_macros::server]
pub async fn set_start_here_hidden(hidden: bool) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let (valence, user) = super::require_owner_session().await?;
        save_start_here_hidden(&valence, &user, hidden)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = hidden;
        Err(ServerFnError::new("ssr only"))
    }
}
