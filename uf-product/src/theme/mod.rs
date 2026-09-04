//! Product appearance preferences and per-app brand seeds.
//!
//! Light/dark mode, brand source, and seed color live here as client-side types
//! and helpers. Server load/save for signed-in users is in [`crate::services`].
//! Shell app-bar appearance UI is composed in `uf-integrations`.
//!
//! # Concern → API
//!
//! | Concern | API |
//! |---------|-----|
//! | Read / write prefs in the reactive tree | [`use_appearance_preferences`], [`AppearanceContext`] |
//! | Anonymous prefs in `localStorage` | [`read_local_appearance`], [`write_local_appearance`], [`APPEARANCE_STORAGE_KEY`] |
//! | Push prefs onto the theme signal | [`apply_appearance_preferences`], [`resolve_effective_brand_seed`] |
//! | Product / shell brand hex | [`PRODUCT_BRAND_PRESETS`], [`brand_seed_for_app_id`], [`SHELL_BRAND_SEED`] |
//!
//! # Example
//!
//! ```rust,ignore
//! use leptos::prelude::*;
//! use uf_product::theme::{
//!     apply_appearance_preferences, use_appearance_preferences, AppearancePreferences,
//! };
//! use orbital_theme::ThemeInjection;
//!
//! let theme = ThemeInjection::use_rw_theme();
//! if let Some(prefs) = use_appearance_preferences() {
//!     apply_appearance_preferences(theme, &prefs.get(), "#1a6f94");
//! } else {
//!     apply_appearance_preferences(theme, &AppearancePreferences::light_product(), "#1a6f94");
//! }
//! ```
//!
//! Detailed host wiring: `uf-product/examples/auth_shell_host` and
//! [`crate::telemetry::AppearanceThemeController`].

mod appearance;
mod product_brands;

pub use appearance::{
    apply_appearance_preferences, read_local_appearance, resolve_effective_brand_seed,
    use_appearance_preferences, write_local_appearance, AppearanceContext, AppearancePreferences,
    APPEARANCE_STORAGE_KEY,
};
pub use product_brands::{
    brand_seed_for_app_id, product_avatar_letter, PRODUCT_BRAND_PRESETS, SHELL_BRAND_SEED,
    UF_SHELL_BRAND_SEED,
};
