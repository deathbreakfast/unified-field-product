//! Featured apps catalog service (SSR-only).
//!
//! Domain layer for Valence-backed `/welcome` featured rows. Server functions
//! (Gauge `WelcomeAdmin` + System Valence) live beside other welcome server fns.

mod error;
mod service;

pub use error::FeaturedError;
pub use service::{add, clear_all, list, remove, reorder, FeaturedAppRow};
