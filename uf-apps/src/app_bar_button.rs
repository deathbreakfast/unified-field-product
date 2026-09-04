//! Apps directory chrome — app-bar Apps control.
//!
//! [`ensure_linked`] (re-exported as [`crate::ensure_app_bar_linked`]) registers
//! the default Apps button in the product app-bar utility strip. Mount
//! [`crate::UfAppsRoutes`] separately for `/apps` pages; the button only appears
//! after `ensure_app_bar_linked()`.

use leptos::prelude::*;
use uf_product::primitives::{Button, ButtonAppearance, Tooltip};
use uf_product::{register_app_bar_utility, AppBarUtilityContribution};

use crate::apps_launcher::AppsLauncher;

/// Sort order for the default Apps control in the app-bar utilities pack.
pub const APP_BAR_UTILITY_ORDER: u8 = 20;

/// Apps control for the product app bar. Opens the apps search launcher.
#[component]
pub fn AppBarAppsButton() -> impl IntoView {
    let open = RwSignal::new(false);

    view! {
        <Tooltip content="Apps">
            <div data-testid="app-bar-apps">
                <Button
                    appearance=ButtonAppearance::Subtle
                    icon=icondata::AiAppstoreOutlined
                    attr:aria-label="Apps"
                    on_click=Callback::new(move |_| {
                        open.set(true);
                    })
                />
            </div>
        </Tooltip>
        <AppsLauncher open=open />
    }
}

fn render_apps_utility() -> AnyView {
    view! { <AppBarAppsButton /> }.into_any()
}

inventory::submit! {
    AppBarUtilityContribution::new(APP_BAR_UTILITY_ORDER, "apps", render_apps_utility)
}

/// Ensure this crate's inventory submissions are linked (call from hosts if needed).
///
/// Registers the app-bar Apps utility ([`AppBarAppsButton`] + [`crate::apps_launcher::AppsLauncher`]).
/// Does not mount [`crate::UfAppsRoutes`]; add those routes in the host router.
pub fn ensure_linked() {
    register_app_bar_utility();
}
