//! Product shell layout: Orbital `Layout` plus app-bar / left-nav slots and a
//! permission toast bus.
//!
//! Session loading lives in `uf-product` (`get_session` / `init_auth_resource`).
//! Auth menu contents are host-provided via [`crate::provide_shell_auth_menu`].
//! Help step inventory content comes from app crates; with `offering-help` this
//! layout mounts [`uf_help::HelpTourPlayer`] by default.
//!
//! # Concern → API
//!
//! | Concern | API |
//! |---------|-----|
//! | Wrap page content in product chrome | [`UnifiedFieldShellLayout`] |
//! | Fill app-bar / left-nav regions | [`ShellAppBar`], [`ShellLeftNav`] |
//! | Tell the app bar to show the sidebar hamburger | [`ShellSidebarToggle`] |
//!
//! # Example
//!
//! See the crate-root Getting started block and
//! `examples/shell-chrome-host`. Mid-level search wiring lives on
//! [`crate::SearchSourcePicker`].
//!
//! When the `offering-help` feature is enabled, [`UnifiedFieldShellLayout`]
//! mounts [`uf_help::HelpTourPlayer`] beside the layout so hosts do not wire
//! the player by hand.
use leptos::prelude::*;
use uf_product::components::{
    AppBarDensity, Layout, LayoutHeader, LayoutMain, LayoutSidebar, SidebarPresentation,
};
use uf_product::primitives::ToasterProvider;
use uf_product::services::permission_server_errors::{
    provide_permission_toast_bus, PermissionToastRequest,
};
use uf_product::{provide_access_gate_state, provide_auth_dialog_controller};

/// When true, [`crate::UnifiedFieldAppBar`] should render the sidebar hamburger control.
#[derive(Clone, Copy)]
pub struct ShellSidebarToggle(pub bool);

/// Slot for the app bar region of [`UnifiedFieldShellLayout`].
#[slot]
pub struct ShellAppBar {
    /// The app bar content, typically a [`UnifiedFieldAppBar`](crate::UnifiedFieldAppBar).
    pub children: Children,
}

/// Slot for the left navigation column of [`UnifiedFieldShellLayout`].
#[slot]
pub struct ShellLeftNav {
    /// The sidebar navigation content.
    pub children: ChildrenFn,
}

#[component]
fn PermissionToastListener(
    /// Two-way signal holding the permission toast request.
    permission_toast_request: RwSignal<Option<PermissionToastRequest>>,
) -> impl IntoView {
    // Gauge-backed "Request Permission" navigation is unavailable until gauge
    // is git-standalone; keep the listener mount so the toast bus still clears.
    let _ = permission_toast_request;

    view! { <></> }
}

/// Unified Field product shell layout (Orbital v0.1.2 `Layout` + scroll-behind AppBar).
#[component]
pub fn UnifiedFieldShellLayout(
    /// Optional shell app bar.
    #[prop(optional)]
    shell_app_bar: Option<ShellAppBar>,
    /// Optional shell left nav.
    #[prop(optional)]
    shell_left_nav: Option<ShellLeftNav>,
    /// Child content rendered inside the component.
    children: Children,
) -> impl IntoView {
    let permission_toast_request = RwSignal::new(None::<PermissionToastRequest>);
    provide_permission_toast_bus(permission_toast_request.write_only());
    // Shared with AppBarUserMenu + RequireAuthenticated so Sign In opens the modal.
    let _auth_dialog = provide_auth_dialog_controller();
    // Shared with RequireAuthenticated + HelpTourPlayer so tours skip the gate.
    let _access_gate = provide_access_gate_state();
    // Closed by default so Auto overlay mode does not open a drawer on first paint.
    let sidebar_open = RwSignal::new(false);
    let show_sidebar_toggle = shell_left_nav.is_some();
    provide_context(ShellSidebarToggle(show_sidebar_toggle));

    view! {
        <ToasterProvider>
            <PermissionToastListener permission_toast_request />
            {match (shell_app_bar, shell_left_nav) {
                (Some(ShellAppBar { children: header_children }), Some(ShellLeftNav { children: sidebar_children })) => {
                    view! {
                        <Layout
                            overlay_header=true
                            header_inset=AppBarDensity::Compact
                            data_testid="unified-field-shell-layout"
                            sidebar_open=sidebar_open
                            sidebar_presentation=SidebarPresentation::Auto
                            layout_header=LayoutHeader { children: header_children }
                            layout_sidebar=LayoutSidebar { children: sidebar_children }
                            layout_main=LayoutMain { children }
                        />
                    }.into_any()
                }
                (Some(ShellAppBar { children: header_children }), None) => {
                    view! {
                        <Layout
                            overlay_header=true
                            header_inset=AppBarDensity::Compact
                            data_testid="unified-field-shell-layout"
                            layout_header=LayoutHeader { children: header_children }
                            layout_main=LayoutMain { children }
                        />
                    }.into_any()
                }
                (None, Some(ShellLeftNav { children: sidebar_children })) => {
                    view! {
                        <Layout
                            overlay_header=true
                            header_inset=AppBarDensity::Compact
                            data_testid="unified-field-shell-layout"
                            sidebar_open=sidebar_open
                            sidebar_presentation=SidebarPresentation::Auto
                            layout_sidebar=LayoutSidebar { children: sidebar_children }
                            layout_main=LayoutMain { children }
                        />
                    }.into_any()
                }
                (None, None) => {
                    view! {
                        <Layout
                            overlay_header=true
                            header_inset=AppBarDensity::Compact
                            data_testid="unified-field-shell-layout"
                            layout_main=LayoutMain { children }
                        />
                    }.into_any()
                }
            }}
            {
                #[cfg(feature = "offering-help")]
                {
                    uf_help::ensure_linked();
                    view! { <uf_help::HelpTourPlayer /> }.into_any()
                }
                #[cfg(not(feature = "offering-help"))]
                {
                    view! { <></> }.into_any()
                }
            }
        </ToasterProvider>
    }
}
