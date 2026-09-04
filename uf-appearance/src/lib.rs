//! Optional Appearance product offering — desktop app-bar appearance popover.
//!
//! Ships [`AppBarAppearanceButton`] for dark/light toggle and a shortcut to user
//! appearance settings. Preference storage and Valence services stay in `uf-product`;
//! compact avatar-menu appearance rows also live there.
//!
//! ## Features
//!
//! - **App-bar appearance control** — Popover with dark mode switch and settings link for
//!   the stock Unified Field app bar. Call [`ensure_linked`] once at host boot to register
//!   the inventory contribution, or mount [`AppBarAppearanceButton`] in custom chrome.
//!   [Get started](#getting-started)
//!
//! ## Getting started
//!
//! The Appearance control writes dark mode through [`uf_product::app_bar_dark_mode_bind`] and
//! navigates to [`uf_product::paths::USER_APPEARANCE`] for full preference editing. Call
//! [`ensure_linked`] once at host boot before shell chrome renders when you want the stock
//! app-bar utilities pack to include Appearance.
//!
//! **Prerequisites:** `ssr` and/or `hydrate` on this crate and host deps; `uf-product`
//! app-bar utilities registry. Alternatively enable `uf-integrations` feature
//! `offering-appearance` or `full` so the default utilities pack links this inventory.
//!
//! ```rust,ignore
//! use leptos::prelude::*;
//! use uf_appearance::{ensure_linked, AppBarAppearanceButton};
//! use uf_product::app_bar_dark_mode_bind;
//!
//! // Once at host boot, before UnifiedFieldShellLayout renders:
//! ensure_linked();
//!
//! // Or mount the control in custom chrome:
//! let dark = app_bar_dark_mode_bind();
//! view! { <AppBarAppearanceButton /> };
//! assert!(dark.get_untracked(), "dark mode bind is live after mount");
//! ```
//!
//! On success the stock app bar shows the Appearance utility (order
//! [`APP_BAR_UTILITY_ORDER`]) and the popover toggles dark mode. Runnable reference:
//! `cargo check -p shell-chrome-host --features ssr`.
//!
//! **Failure modes:** Without [`ensure_linked`] (and without `offering-appearance` /
//! `full`), inventory submissions are not linked and the control never appears in the
//! default utilities pack. Custom chrome that skips `ensure_linked` must mount
//! [`AppBarAppearanceButton`] directly.
//!
//! ## Examples
//!
//! Start with [`ensure_linked`] + [`AppBarAppearanceButton`] in
//! [Getting started](#getting-started). Default app-bar utilities pack via `uf-integrations`
//! `offering-appearance`. Full shell with appearance control: `examples/shell-chrome-host`.
//!
//! ```bash
//! cargo check -p shell-chrome-host --features ssr
//! ```
//!
//! ## Where to look next
//!
//! - [`AppBarAppearanceButton`] — popover UI and dark-mode switch.
//! - [`ensure_linked`] — inventory link for the stock utilities pack.
//! - `uf_product::app_bar_dark_mode_bind` — shared dark-mode signal for the app bar.
//! - `uf_product::paths::USER_APPEARANCE` — full appearance preferences page.
//! - `uf-integrations` (`offering-appearance` / `full`) — wires this crate into shell chrome.
//! - `examples/shell-chrome-host` — teaching host (`cargo check -p shell-chrome-host --features ssr`).

#![allow(missing_docs)]

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;
use uf_product::app_bar_dark_mode_bind;
use uf_product::paths::USER_APPEARANCE;
use uf_product::primitives::{
    Button, ButtonAppearance, Divider, Flex, FlexGap, Popover, PopoverPosition, PopoverSize,
    PopoverTrigger, PopoverTriggerType, Switch,
};
use uf_product::{register_app_bar_utility, AppBarUtilityContribution};

/// Sort order for the default Appearance control in the app-bar utilities pack.
pub const APP_BAR_UTILITY_ORDER: u8 = 30;

/// Appearance popover for the Unified Field app bar — dark / light mode and settings link.
///
/// Call [`ensure_linked`] at startup for the stock utilities pack, or mount this component
/// directly in custom chrome.
#[component]
pub fn AppBarAppearanceButton() -> impl IntoView {
    let navigate = use_navigate();
    let dark = app_bar_dark_mode_bind();

    let open_appearance_settings = Callback::new(move |_| {
        navigate(USER_APPEARANCE, NavigateOptions::default());
    });

    view! {
        <Popover
            trigger_type=PopoverTriggerType::Click
            position=PopoverPosition::BottomEnd
            size=PopoverSize::Small
        >
            <PopoverTrigger slot>
                <div data-testid="app-bar-appearance">
                    <Button
                        appearance=ButtonAppearance::Subtle
                        icon=icondata::AiBgColorsOutlined
                        attr:aria-label="Appearance"
                    />
                </div>
            </PopoverTrigger>
            <Flex vertical=true gap=FlexGap::Small>
                <div data-testid="app-bar-appearance-menu">
                    <Switch bind=dark label="Dark mode" />
                </div>
                <Divider />
                <div data-testid="app-bar-appearance-settings-link">
                    <Button
                        appearance=ButtonAppearance::Subtle
                        on_click=open_appearance_settings
                    >
                        "Appearance settings"
                    </Button>
                </div>
            </Flex>
        </Popover>
    }
}

fn render_appearance_utility() -> AnyView {
    view! { <AppBarAppearanceButton /> }.into_any()
}

inventory::submit! {
    AppBarUtilityContribution::new(APP_BAR_UTILITY_ORDER, "appearance", render_appearance_utility)
}

/// Ensure this crate's inventory submissions are linked (call from hosts if needed).
///
/// Without this call (or `uf-integrations` `offering-appearance` / `full`), the Appearance
/// control does not appear in the default app-bar utilities pack.
pub fn ensure_linked() {
    register_app_bar_utility();
}
