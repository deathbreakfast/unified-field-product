//! Featured apps catalog service (SSR-only).
//!
//! Domain layer for Valence-backed `/welcome` featured rows and their change
//! history. Server functions (Gauge `WelcomeAdmin` + session Valence) live beside
//! other welcome server fns.

mod error;
mod history;
mod service;

pub use error::FeaturedError;
#[cfg(feature = "e2e-seed")]
pub use history::clear_history;
pub use history::{
    current_app_ids, get_change, latest_change, list_changes, publish, record_change, record_since,
    restore, FeaturedChangeRow, PublishOutcome,
};
pub use service::{add, app_name, clear_all, list, remove, reorder, replace_all, FeaturedAppRow};
