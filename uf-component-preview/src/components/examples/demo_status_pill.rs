//! Teaching widget that shows how `#[component_doc]` feeds the `/orbital` catalog.
//!
//! After `preview_registrations!` + `.extend(examples::all())` in the preview registry,
//! this page appears under left nav and at `/orbital/demo-status-pill`.

use leptos::prelude::*;
use orbital_core_components::{Badge, BadgeAppearance, BadgeColor};
use orbital_macros::component_doc;

/// Compact status label used only to teach preview registration.
///
/// # Examples
///
/// ## Ready
/// Filled brand badge for a short status word.
/// <!-- preview -->
/// ```rust
/// view! {
///     <div data-testid="demo-status-pill-preview">
///         <DemoStatusPill label="Ready".to_string() />
///     </div>
/// }
/// ```
#[component_doc(
    section = "Examples",
    category = "Unified Field",
    preview_slug = "demo-status-pill",
    preview_label = "Demo Status Pill"
)]
#[component]
pub fn DemoStatusPill(
    /// Short status text shown inside the badge.
    #[prop(into)]
    label: String,
) -> impl IntoView {
    view! {
        <Badge appearance=BadgeAppearance::Filled color=BadgeColor::Brand>
            {label}
        </Badge>
    }
}
