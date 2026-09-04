//! Applies user appearance preferences to the active Orbital theme on navigation.

use crate::theme::{
    apply_appearance_preferences, read_local_appearance, AppearanceContext, AppearancePreferences,
};
use leptos::prelude::*;
use leptos_router::hooks::use_location;
use orbital_theme::ThemeInjection;

use super::page_view_tracker::{resolve_app_for_path, UfAppRouteEntry};

/// Keeps theme mode and brand color in sync with user prefs and the current product route.
#[component]
pub fn AppearanceThemeController(
    /// Route paths this applies to.
    routes: &'static [UfAppRouteEntry],
) -> impl IntoView {
    let theme = ThemeInjection::use_rw_theme();
    let location = use_location();

    Effect::new(move |_| {
        let path = location.pathname.get();
        let prefs = AppearanceContext::use_ctx()
            .map(|ctx| ctx.prefs.get())
            .or_else(read_local_appearance)
            .unwrap_or_else(AppearancePreferences::light_product);

        let entry = resolve_app_for_path(&path, routes);
        apply_appearance_preferences(theme, &prefs, entry.brand_seed);
    });

    view! { <></> }
}
