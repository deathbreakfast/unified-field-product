//! Compact AppBar extras for host `lepton_shell::AppBarUserMenu`.
//!
//! On narrow viewports the product app bar hides trailing utilities and provides a
//! [`Signal`] so the avatar / login menu can render appearance rows **inside**
//! the Menu tree (where [`MenuItem`] context exists). Do not build those rows via
//! [`Callback`] from outside the Menu — that runs under the wrong reactive owner.

use leptos::prelude::*;
use orbital_theme::{use_theme_mode, ThemeMode};

use crate::primitives::{Divider, MenuItem, Switch};
use crate::services::save_my_appearance;
use crate::theme::{use_appearance_preferences, write_local_appearance};

/// Context: when true, the host auth menu embeds compact extras in the login menu.
#[derive(Clone, Copy)]
pub struct AppBarMenuExtrasInjection {
    /// Show appearance rows at the top of the avatar menu.
    pub show: Signal<bool>,
}

/// Provide compact-menu extras flag for the current reactive tree.
pub fn provide_app_bar_menu_extras(show: Signal<bool>) {
    provide_context(AppBarMenuExtrasInjection { show });
}

/// Read optional app-bar menu extras flag from context.
pub fn use_app_bar_menu_extras() -> Option<AppBarMenuExtrasInjection> {
    use_context::<AppBarMenuExtrasInjection>()
}

fn use_dark_mode_bind() -> RwSignal<bool> {
    let (mode, set_mode) = use_theme_mode();
    let appearance_prefs = use_appearance_preferences();
    let dark = RwSignal::new(mode.get_untracked() == ThemeMode::Dark);

    Effect::new(move |_| {
        dark.set(mode.get() == ThemeMode::Dark);
    });

    Effect::new(move |prev: Option<bool>| {
        let is_dark = dark.get();
        if prev.is_some() && prev != Some(is_dark) {
            let new_mode = if is_dark {
                ThemeMode::Dark
            } else {
                ThemeMode::Light
            };
            set_mode(new_mode);
            let color_mode = if is_dark { "dark" } else { "light" };
            if let Some(ctx) = appearance_prefs {
                ctx.update(|prefs| prefs.color_mode = color_mode.to_string());
                let prefs_snapshot = ctx.get_untracked();
                write_local_appearance(&prefs_snapshot);
                leptos::task::spawn_local(async move {
                    let brand_source = prefs_snapshot.brand_source.clone();
                    let brand_seed = prefs_snapshot.brand_seed_color.clone();
                    let _ =
                        save_my_appearance(color_mode.to_string(), brand_source, brand_seed).await;
                });
            }
        }
        is_dark
    });

    dark
}

/// Appearance controls for embedding in the compact AppBar user menu.
#[component]
pub fn AppBarAppearanceMenuItems() -> impl IntoView {
    let dark = use_dark_mode_bind();

    view! {
        <div data-testid="app-bar-appearance-menu">
            <Switch bind=dark label="Dark mode" />
        </div>
        <div data-testid="app-bar-appearance-settings-link">
            <MenuItem value="appearance_settings">
                "Appearance settings"
            </MenuItem>
        </div>
        <Divider />
    }
}

/// Compact trailing utilities folded into the avatar / login menu.
///
/// Only ship rows that work today — omit Search / Help / Apps until those surfaces
/// have real compact implementations.
#[component]
pub fn AppBarCompactMenuExtras() -> impl IntoView {
    view! {
        <AppBarAppearanceMenuItems />
    }
}

/// Shared dark-mode bind for the desktop appearance popover (uf-integrations).
pub fn app_bar_dark_mode_bind() -> RwSignal<bool> {
    use_dark_mode_bind()
}
