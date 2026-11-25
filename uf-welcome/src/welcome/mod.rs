//! The welcome app itself: cards, layout, pages, and server data.

/// Reusable cards shown on the welcome page.
pub mod components;
/// Plain-words descriptions of featured list changes.
pub mod describe;
/// Featured apps Valence catalog (SSR).
#[cfg(feature = "ssr")]
pub mod featured;
/// The welcome app's shell layout.
pub mod layout;
/// Welcome and admin pages.
pub mod pages;
/// Server functions backing the welcome page's cards and admin.
pub mod server;

pub use components::{
    FeaturedAppsCard, FeaturedAppsCardBody, PopularAppsCard, PopularAppsCardBody, RecentAppsCard,
    RecentAppsCardBody, WelcomeCard,
};
pub use layout::WelcomeLayout;
pub use pages::{WelcomeAdminPage, WelcomePage};
pub use server::{
    clear_my_history, get_featured_apps, get_pinned_apps, get_popular_apps, get_recent_apps,
    get_recent_pages, get_waiting_on_me, get_welcome_settings, pin_app, reorder_pinned_apps,
    set_start_here_hidden, unpin_app, AppLinkDto, RecentApp, RecentPageDto, WelcomeSettingsDto,
};
