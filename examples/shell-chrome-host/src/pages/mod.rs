//! Home, coming-soon, and 404 demo pages for the shell chrome host.
#![allow(missing_docs)]

mod coming_soon;
mod home;
mod not_found;

pub use coming_soon::ComingSoonDemoPage;
pub use home::HomePage;
pub use not_found::NotFoundDemoPage;
