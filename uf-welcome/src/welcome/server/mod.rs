//! Server functions and DTOs for welcome cards and featured admin.

mod featured;
mod usage;

#[cfg(feature = "ssr")]
pub use featured::E2E_WELCOME_ADMIN_SESSION_KEY;
pub use featured::{
    add_featured_app, can_manage_welcome_featured, get_featured_apps, list_manageable_apps,
    remove_featured_app, reorder_featured_apps, AppLinkDto, ManageableAppDto,
};
pub use usage::{get_my_most_used, get_popular_apps, get_recent_apps};

/// Back-compat alias for the recent-apps card DTO.
pub type RecentApp = AppLinkDto;
