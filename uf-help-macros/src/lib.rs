//! Proc macros for Help spotlight tour step registration.
//!
//! Expands [`help_spotlight_step`] into `inventory::submit!` entries consumed by
//! `uf-help`. Runtime tour player, visit progress, and shell mount live in `uf-help`
//! and `uf-integrations` (`offering-help`).
//!
//! ## Features
//!
//! - **Spotlight step macro** — Registers a Leptos `#[component]` as a Help tour step
//!   with route path, stable `feature_highlight` key, optional DOM cutout id, and panel
//!   copy. [Get started](#help-spotlight-step)
//!
//! ## Help spotlight step
//!
//! [`help_spotlight_step`] keeps your original Leptos component and emits a
//! `uf_help::HelpStepDescriptor` through `inventory::submit!`. App crates collect steps
//! in `help_steps.rs` modules and call `ensure_help_steps_linked()` so registrations
//! survive linker dead-stripping in the host binary.
//!
//! **Prerequisites:** `uf-help-macros` and `uf-help` in the app crate; matching HTML
//! `id` on the spotlight target when `spotlight` is set.
//!
//! ```rust,ignore
//! use leptos::prelude::*;
//! use uf_help_macros::help_spotlight_step;
//!
//! #[help_spotlight_step(
//!     route = "/apps",
//!     feature_highlight = "apps-search",
//!     title = "Search apps",
//!     spotlight = "apps-search-input",
//!     position = "bottom",
//!     order = 10,
//! )]
//! #[component]
//! pub fn AppsSearchHelp() -> impl IntoView {
//!     view! {
//!         <input id="apps-search-input" />
//!         <p data-testid="help-step-apps-search">
//!             "Use search to find installed apps by name."
//!         </p>
//!     }
//! }
//!
//! /// Call from the app crate so inventory is linked into the host binary.
//! pub fn ensure_help_steps_linked() {}
//! ```
//!
//! On success the step registers in `inventory` and appears when `uf_help` collects
//! `HelpStepDescriptor` rows for `/apps`. Keep `feature_highlight` stable; renaming
//! creates a new highlight key for returning users. See `uf-apps/src/help_steps.rs` for
//! a full route module and the `uf-help` **Author a spotlight step** section for DOM
//! anchors, force-link, and shell mount.
//!
//! ## Where to look next
//!
//! - [`help_spotlight_step`] — attribute table (`route`, `feature_highlight`, `spotlight`, …).
//! - `uf-help` — runtime inventory, tour player, visit progress, app-bar Help.
//! - `uf-integrations` — `offering-help` mounts tour player on stock shell layout.

#![allow(missing_docs)]

use proc_macro::TokenStream;

mod help_spotlight_step;

/// Register a Leptos component as a Help spotlight tour step.
///
/// Keeps the original `#[component]` fn and submits a
/// `uf_help::HelpStepDescriptor` via `inventory`.
///
/// | Attribute | Required | Notes |
/// |-----------|----------|-------|
/// | `route` | yes | Exact route path, e.g. `"/apps"`, or `"/apps/:app_name"` for one-segment app overviews |
/// | `feature_highlight` | yes | Stable per-step identity (seen / replay key); do not rename in place |
/// | `title` | no | Panel header; defaults to `feature_highlight` |
/// | `spotlight` | no | DOM element `id` for the cutout; omit to center in the viewport |
/// | `position` | no | Popover placement: `top` (default), `bottom`, `left`, `right`, or `*-start` / `*-end` |
/// | `order` | no | Sort key within the route (default `0`) |
///
/// # Examples
///
/// Pair the macro with a matching HTML `id` on the spotlight target and a
/// `data-testid` on the step body for Playwright (see `uf-apps/src/help_steps.rs`).
///
/// ```rust,ignore
/// use leptos::prelude::*;
/// use uf_help_macros::help_spotlight_step;
///
/// #[help_spotlight_step(
///     route = "/apps",
///     feature_highlight = "apps-search",
///     title = "Search apps",
///     spotlight = "apps-search-input",
///     position = "bottom",
///     order = 10,
/// )]
/// #[component]
/// pub fn AppsSearchHelp() -> impl IntoView {
///     view! {
///         <input id="apps-search-input" />
///         <p data-testid="help-step-apps-search">
///             "Use search to find installed apps by name."
///         </p>
///     }
/// }
///
/// /// Call from the app crate so inventory is linked into the host binary.
/// pub fn ensure_help_steps_linked() {}
/// ```
#[proc_macro_attribute]
pub fn help_spotlight_step(attr: TokenStream, input: TokenStream) -> TokenStream {
    help_spotlight_step::expand(attr.into(), input.into()).into()
}
