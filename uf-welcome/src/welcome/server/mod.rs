//! Server functions and DTOs for welcome cards and featured admin.

mod featured;
mod pinned;
mod settings;
mod usage;
mod waiting;

#[cfg(feature = "ssr")]
pub use featured::E2E_WELCOME_ADMIN_SESSION_KEY;
pub use featured::{
    add_featured_app, can_manage_welcome_featured, get_featured_admin, get_featured_apps,
    list_featured_history, list_manageable_apps, publish_featured, remove_featured_app,
    reorder_featured_apps, restore_featured, AppLinkDto, FeaturedAdminAppDto, FeaturedAdminDto,
    FeaturedChangeDto, ManageableAppDto, PublishFeaturedResult,
};
pub use pinned::{get_pinned_apps, pin_app, reorder_pinned_apps, unpin_app};
#[cfg(feature = "ssr")]
pub use pinned::{list_pinned, pin, reorder_pinned, unpin, PinnedError};
pub use settings::{get_welcome_settings, set_start_here_hidden, WelcomeSettingsDto};
#[cfg(feature = "ssr")]
pub use settings::{load_settings, save_start_here_hidden, SettingsError};
pub use usage::{
    clear_my_history, get_popular_apps, get_recent_apps, get_recent_pages, RecentPageDto,
};
#[cfg(feature = "ssr")]
pub use waiting::waiting_for_user;
pub use waiting::{get_waiting_on_me, WAITING_LIMIT};

/// Back-compat alias for the recent-apps card DTO.
pub type RecentApp = AppLinkDto;

/// Session Valence and user record for owner-only Welcome data, or `None`
/// when nobody is signed in. Never elevates the actor.
#[cfg(feature = "ssr")]
async fn owner_session(
) -> Result<Option<(valence::Valence, valence::RecordId)>, leptos::prelude::ServerFnError> {
    let ctx = uf_product::ssr::higgs().await?;
    let Some(session_user) = ctx.session_user_id() else {
        return Ok(None);
    };
    let user = uf_product::ssr::session_user_record_id(session_user)?;
    let valence = uf_product::ssr::valence(&ctx)?;
    Ok(Some((valence, user)))
}

/// [`owner_session`] for writes: signed-out callers get an error.
#[cfg(feature = "ssr")]
async fn require_owner_session(
) -> Result<(valence::Valence, valence::RecordId), leptos::prelude::ServerFnError> {
    owner_session()
        .await?
        .ok_or_else(|| leptos::prelude::ServerFnError::new("sign in to change your Welcome page"))
}
