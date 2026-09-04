//! Apps directory pages (`/apps`, `/apps/:app_name`).

pub mod detail;
pub mod index;

pub use detail::AppDetailPage;
pub use index::AppsIndexPage;
