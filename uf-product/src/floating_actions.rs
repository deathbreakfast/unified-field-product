//! Inventory contributions for the shell's floating actions menu (FAM).
//!
//! Apps submit [`FloatingActionContribution`] rows the same way offerings submit
//! app-bar utilities. A host opts in by passing `ShellFloatingActions` to
//! `uf-integrations` `UnifiedFieldShellLayout`, which mounts
//! [`RegisteredFloatingActions`]. A host that leaves the slot out gets no FAB.
//!
//! # Registering an item
//!
//! ```rust,ignore
//! use leptos::prelude::*;
//! use uf_product::primitives::FloatingActionsMenuItem;
//! use uf_product::FloatingActionContribution;
//!
//! fn render_discuss() -> AnyView {
//!     view! {
//!         <FloatingActionsMenuItem
//!             tooltip="Discuss".to_string()
//!             icon=icondata::AiCommentOutlined
//!             testid="fam-discuss"
//!         />
//!     }
//!     .into_any()
//! }
//!
//! inventory::submit! {
//!     FloatingActionContribution::new(10, "discuss", render_discuss)
//! }
//!
//! /// Call from the host so the linker keeps the submission.
//! pub fn ensure_fam_linked() {
//!     uf_product::register_floating_action();
//! }
//! ```

use leptos::prelude::*;

use crate::primitives::{FloatingActionsMenu, FloatingActionsMenuConfig};

/// Accessible name of the menu when the host passes no config.
pub const DEFAULT_FLOATING_ACTIONS_LABEL: &str = "Page actions";

/// One item an app contributes to the shell's floating actions menu.
pub struct FloatingActionContribution {
    /// Sort key (lower first).
    pub order: u8,
    /// Stable id for tests and docs.
    pub id: &'static str,
    /// Render the item, usually a `FloatingActionsMenuItem`. The app may render
    /// its own drawer or overlay beside the item.
    pub render: fn() -> AnyView,
}

impl FloatingActionContribution {
    /// Construct a contribution for inventory registration.
    pub const fn new(order: u8, id: &'static str, render: fn() -> AnyView) -> Self {
        Self { order, id, render }
    }
}

inventory::collect!(FloatingActionContribution);

/// No-op touch point so app crates can force-link their inventory row into the binary.
pub fn register_floating_action() {}

/// Collect app contributions sorted by [`FloatingActionContribution::order`].
pub fn collect_floating_actions() -> Vec<&'static FloatingActionContribution> {
    let mut items: Vec<&'static FloatingActionContribution> =
        inventory::iter::<FloatingActionContribution>
            .into_iter()
            .collect();
    items.sort_by_key(|c| c.order);
    items
}

/// Viewport-pinned bottom-right config labelled [`DEFAULT_FLOATING_ACTIONS_LABEL`].
pub fn default_floating_actions_config() -> FloatingActionsMenuConfig {
    FloatingActionsMenuConfig {
        aria_label: DEFAULT_FLOATING_ACTIONS_LABEL.to_string(),
        ..FloatingActionsMenuConfig::fixed(24.0, 24.0)
    }
}

/// Orbital `FloatingActionsMenu` holding every registered [`FloatingActionContribution`].
///
/// Renders nothing when no app registered an item, so the host never shows an
/// empty FAB.
#[component]
pub fn RegisteredFloatingActions(
    /// Menu config; defaults to [`default_floating_actions_config`].
    #[prop(optional)]
    config: Option<FloatingActionsMenuConfig>,
) -> impl IntoView {
    view! { <FloatingActionsMenuView items=collect_floating_actions() config /> }
}

#[component]
fn FloatingActionsMenuView(
    /// Contributions to render, already sorted.
    items: Vec<&'static FloatingActionContribution>,
    /// Menu config; defaults to [`default_floating_actions_config`].
    config: Option<FloatingActionsMenuConfig>,
) -> impl IntoView {
    if items.is_empty() {
        return ().into_any();
    }
    let config = config.unwrap_or_else(default_floating_actions_config);
    view! {
        <FloatingActionsMenu config>
            {items.into_iter().map(|c| (c.render)()).collect_view()}
        </FloatingActionsMenu>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render_probe() -> AnyView {
        view! { <span data-testid="fam-test-probe">"probe"</span> }.into_any()
    }

    inventory::submit! {
        FloatingActionContribution::new(50, "floating_actions_test_probe", render_probe)
    }

    #[test]
    fn collect_floating_actions_sorted_happy() {
        let items = collect_floating_actions();
        assert!(
            items.iter().any(|c| c.id == "floating_actions_test_probe"),
            "the test-submitted contribution must be collected"
        );
        let mut prev = 0u8;
        for item in &items {
            assert!(item.order >= prev, "contributions must sort by order");
            prev = item.order;
            assert!(!item.id.is_empty());
        }
    }

    // Rendering `AnyView` to HTML needs the `ssr` feature (CI runs
    // `cargo test -p uf-product --features ssr`). The menu creates signals, so it
    // renders under a reactive owner.
    #[cfg(feature = "ssr")]
    fn render_html(view: impl FnOnce() -> AnyView) -> String {
        Owner::new().with(|| view().to_html())
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn floating_actions_menu_renders_items_happy() {
        let html = render_html(|| {
            RegisteredFloatingActions(RegisteredFloatingActionsProps { config: None }).into_any()
        });
        assert!(
            html.contains(r#"role="group""#) && html.contains(DEFAULT_FLOATING_ACTIONS_LABEL),
            "expected the labelled menu group, got: {html}"
        );
        assert!(
            html.contains("floating-actions-menu-trigger"),
            "expected the FAB trigger, got: {html}"
        );
        assert!(
            html.contains("fam-test-probe"),
            "expected the registered probe item, got: {html}"
        );
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn floating_actions_menu_empty_renders_nothing_sad() {
        let html = render_html(|| {
            FloatingActionsMenuView(FloatingActionsMenuViewProps {
                items: Vec::new(),
                config: None,
            })
            .into_any()
        });
        assert!(
            !html.contains("floating-actions-menu-trigger") && !html.contains(r#"role="group""#),
            "no registered items must render no FAB, got: {html}"
        );
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn floating_actions_menu_custom_config_happy() {
        let html = render_html(|| {
            let config = FloatingActionsMenuConfig {
                aria_label: "Host actions".to_string(),
                ..FloatingActionsMenuConfig::fixed(16.0, 16.0)
            };
            RegisteredFloatingActions(RegisteredFloatingActionsProps {
                config: Some(config),
            })
            .into_any()
        });
        assert!(
            html.contains("Host actions"),
            "expected the host label, got: {html}"
        );
        assert!(
            !html.contains(DEFAULT_FLOATING_ACTIONS_LABEL),
            "the default label must not render when the host passes a config, got: {html}"
        );
    }
}
