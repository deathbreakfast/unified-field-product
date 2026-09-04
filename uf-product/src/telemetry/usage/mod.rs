//! Aggregate `uf_apps_page_view_log` into welcome-style app shortcuts.
//!
//! Pure ranking and dedupe of page-view rows, queried via `spectra::Spectra`.
//! Welcome card UI lives in `uf-welcome`.
//!
//! # Concern → API
//!
//! | Concern | API |
//! |---------|-----|
//! | Options (top-N, lookback) | [`crate::telemetry::usage::UsageQueryOptions`] |
//! | Per-user recent apps | [`crate::telemetry::usage::recent_apps_for_viewer`] |
//! | Per-user most used | [`crate::telemetry::usage::most_used_for_viewer`] |
//! | Fleet popular apps | [`crate::telemetry::usage::popular_apps`] |
//! | Pure aggregates (tests / custom sources) | [`crate::telemetry::usage::aggregate_recent`], [`crate::telemetry::usage::aggregate_most_used`], [`crate::telemetry::usage::aggregate_popular`] |
//! | Errors | [`crate::telemetry::usage::UsageQueryError`] |
//!
//! # Example
//!
//! ```rust,ignore
//! use spectra::Spectra;
//! use uf_product::telemetry::usage::{recent_apps_for_viewer, UsageQueryOptions};
//!
//! async fn load(spectra: &Spectra, viewer_key: &str) -> Result<Vec<_>, _> {
//!     recent_apps_for_viewer(spectra, viewer_key, &UsageQueryOptions::default()).await
//! }
//! ```

mod aggregate;
mod error;
mod query;
mod viewer;

pub use aggregate::{
    aggregate_most_used, aggregate_popular, aggregate_recent, VisitRow, PAGE_VIEW_LOG_TABLE,
};
pub use error::UsageQueryError;
pub use query::{
    most_used_for_viewer, most_used_for_viewer_on, popular_apps, popular_apps_on,
    recent_apps_for_viewer, recent_apps_for_viewer_on, resolve_usage_links, UsageAppLink,
    UsageQueryOptions,
};
pub use viewer::{resolve_usage_viewer_key, UsageViewerOverride, E2E_USAGE_VIEWER_SESSION_KEY};
