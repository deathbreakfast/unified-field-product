use leptos::prelude::*;
use orbital_primitives::Switch;
#[cfg(feature = "hydrate")]
use orbital_theme::set_theme_mode;
use orbital_theme::{Theme, ThemeMode};

/// Dark / light toggle for the preview catalog shell AppBar.
#[component]
pub fn PreviewThemeToggle() -> impl IntoView {
    let theme = Theme::use_rw_theme();
    let dark = RwSignal::new(theme.get_untracked().mode == ThemeMode::Dark);

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        dark.set(theme.with(|t| t.mode == ThemeMode::Dark));
    });

    #[cfg(feature = "hydrate")]
    Effect::new(move |prev: Option<bool>| {
        let is_dark = dark.get();
        if prev.is_some() && prev != Some(is_dark) {
            set_theme_mode(
                theme,
                if is_dark {
                    ThemeMode::Dark
                } else {
                    ThemeMode::Light
                },
            );
        }
        is_dark
    });

    view! {
        <div data-testid="preview-theme-toggle">
            <Switch bind=dark label="Dark mode" />
        </div>
    }
}
