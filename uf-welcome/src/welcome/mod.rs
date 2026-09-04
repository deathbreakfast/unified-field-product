//! The welcome app itself: cards, layout, pages, and server data.

/// Reusable cards shown on the welcome page.
pub mod components;
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
    FeaturedAppsCard, FeaturedAppsCardBody, MyMostUsedCard, MyMostUsedCardBody, PopularAppsCard,
    PopularAppsCardBody, RecentAppsCard, RecentAppsCardBody, WelcomeCard,
};
pub use layout::WelcomeLayout;
pub use pages::{WelcomeAdminPage, WelcomePage};
pub use server::{
    get_featured_apps, get_my_most_used, get_popular_apps, get_recent_apps, AppLinkDto, RecentApp,
};
