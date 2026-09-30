//! Demo pages for product UI e2e.
#![allow(missing_docs)]

mod auth;
mod coming_soon;
mod gates;
mod help_fixture;
mod home;
mod not_found;
mod scroll_chrome;
mod utilities_override;
mod workspace_search_hit;

pub use auth::AuthSignInPage;
pub use coming_soon::ComingSoonDemoPage;
pub use gates::{GateEmailPage, GatePermissionAllowPage, GatePermissionPage};
pub use help_fixture::HelpFixtureDeferredPage;
pub use home::HomePage;
pub use not_found::NotFoundDemoPage;
pub use scroll_chrome::ScrollChromePage;
pub use utilities_override::UtilitiesOverridePage;
pub use workspace_search_hit::WorkspaceSearchHitPage;
