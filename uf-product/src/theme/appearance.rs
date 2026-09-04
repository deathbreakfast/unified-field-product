//! User appearance preferences and route-aware brand resolution.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use super::product_brands::SHELL_BRAND_SEED;
use orbital_theme::{set_brand_palette, set_theme_mode, BrandPalette, Theme, ThemeMode};

/// `LocalStorage` key for anonymous appearance preferences.
pub const APPEARANCE_STORAGE_KEY: &str = "orbital.appearance";

/// User-controlled appearance settings mirrored in Valence `user_appearance`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppearancePreferences {
    #[serde(default = "default_color_mode")]
    pub color_mode: String,
    #[serde(default = "default_brand_source")]
    pub brand_source: String,
    pub brand_seed_color: Option<String>,
}

fn default_color_mode() -> String {
    "light".to_string()
}

fn default_brand_source() -> String {
    "product".to_string()
}

impl AppearancePreferences {
    pub fn light_product() -> Self {
        Self {
            color_mode: "light".to_string(),
            brand_source: "product".to_string(),
            brand_seed_color: None,
        }
    }

    pub fn theme_mode(&self) -> ThemeMode {
        if self.color_mode == "dark" {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        }
    }

    pub fn follows_product_brand(&self) -> bool {
        self.brand_source != "custom"
    }
}

/// Resolve the effective brand seed hex from user prefs and the current product brand.
pub fn resolve_effective_brand_seed(
    prefs: &AppearancePreferences,
    product_brand_seed: &str,
) -> String {
    if prefs.follows_product_brand() {
        product_brand_seed.to_string()
    } else {
        prefs
            .brand_seed_color
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| SHELL_BRAND_SEED.to_string())
    }
}

/// Apply appearance preferences and the resolved product brand to the active theme signal.
pub fn apply_appearance_preferences(
    theme: RwSignal<Theme>,
    prefs: &AppearancePreferences,
    product_brand_seed: &str,
) {
    let mode = prefs.theme_mode();
    let current_mode = theme.with_untracked(|t| t.mode);
    if current_mode != mode {
        set_theme_mode(theme, mode);
    }

    let seed = resolve_effective_brand_seed(prefs, product_brand_seed);
    set_brand_palette(theme, BrandPalette { primary: seed });
}

/// Reactive appearance preferences provided at app root.
#[derive(Clone, Copy)]
pub struct AppearanceContext {
    pub prefs: RwSignal<AppearancePreferences>,
}

impl AppearanceContext {
    pub fn provide(initial: AppearancePreferences) -> Self {
        let ctx = Self {
            prefs: RwSignal::new(initial),
        };
        provide_context(ctx);
        ctx
    }

    pub fn use_ctx() -> Option<Self> {
        use_context::<Self>()
    }

    pub fn set_prefs(&self, prefs: AppearancePreferences) {
        self.prefs.set(prefs);
    }
}

pub fn use_appearance_preferences() -> Option<RwSignal<AppearancePreferences>> {
    AppearanceContext::use_ctx().map(|ctx| ctx.prefs)
}

#[cfg(target_arch = "wasm32")]
pub fn read_local_appearance() -> Option<AppearancePreferences> {
    use leptos::web_sys;
    let window = web_sys::window()?;
    let storage = window.local_storage().ok()??;
    let raw = storage.get_item(APPEARANCE_STORAGE_KEY).ok()??;
    serde_json::from_str(&raw).ok()
}

#[cfg(not(target_arch = "wasm32"))]
pub const fn read_local_appearance() -> Option<AppearancePreferences> {
    None
}

#[cfg(target_arch = "wasm32")]
pub fn write_local_appearance(prefs: &AppearancePreferences) {
    use leptos::web_sys;
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(json) = serde_json::to_string(prefs) {
                let _ = storage.set_item(APPEARANCE_STORAGE_KEY, &json);
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub const fn write_local_appearance(_prefs: &AppearancePreferences) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn product_mode_uses_route_brand() {
        let prefs = AppearancePreferences::light_product();
        assert_eq!(resolve_effective_brand_seed(&prefs, "#eaa300"), "#eaa300");
    }

    #[test]
    fn custom_mode_uses_user_seed() {
        let prefs = AppearancePreferences {
            color_mode: "light".to_string(),
            brand_source: "custom".to_string(),
            brand_seed_color: Some("#ff0000".to_string()),
        };
        assert_eq!(resolve_effective_brand_seed(&prefs, "#eaa300"), "#ff0000");
    }

    #[test]
    fn custom_fallback_shell() {
        let prefs = AppearancePreferences {
            color_mode: "light".to_string(),
            brand_source: "custom".to_string(),
            brand_seed_color: None,
        };
        assert_eq!(
            resolve_effective_brand_seed(&prefs, "#eaa300"),
            SHELL_BRAND_SEED
        );
    }
}
