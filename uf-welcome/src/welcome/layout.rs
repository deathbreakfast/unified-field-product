//! The welcome app's shell layout: app bar plus a router [`leptos_router::components::Outlet`] for the welcome page.

use leptos::prelude::*;
use leptos_router::components::Outlet;
use uf_integrations::{
    HostAuthMenu, ShellAppBar, ShellAuthMenu, UnifiedFieldAppBar, UnifiedFieldShellLayout,
};
use uf_product::routes::RequireAuthenticated;

use crate::AppMetadata;

/// Welcome app's shell layout: app bar and a router [`leptos_router::components::Outlet`] for the welcome page.
///
/// Auth menu comes from the host via [`uf_integrations::provide_shell_auth_menu`].
#[component]
pub fn WelcomeLayout() -> impl IntoView {
    let app_name = AppMetadata::name().to_string();
    view! {
        <UnifiedFieldShellLayout>
            <ShellAppBar slot>
                <UnifiedFieldAppBar
                    app_name=app_name
                    app_id=AppMetadata::id()
                    homepage_url="/".to_string()
                >
                    <ShellAuthMenu slot:auth_menu>
                        <HostAuthMenu />
                    </ShellAuthMenu>
                </UnifiedFieldAppBar>
            </ShellAppBar>
            <RequireAuthenticated>
                <Outlet />
            </RequireAuthenticated>
        </UnifiedFieldShellLayout>
    }
}
