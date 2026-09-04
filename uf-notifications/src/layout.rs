use leptos::prelude::*;
use leptos_router::components::Outlet;
use uf_integrations::{
    HostAuthMenu, ShellAppBar, ShellAuthMenu, ShellLeftNav, UnifiedFieldAppBar,
    UnifiedFieldShellLayout,
};
use uf_product::components::{
    Navigation, NavigationBody, NavigationConfig, NavigationLink, NavigationMaterial,
};

use crate::paths;
use crate::AppMetadata;

/// Notifications app's shell layout: app bar, left navigation, and a router
/// [`Outlet`] for the inbox page.
///
/// Auth menu comes from the host via [`uf_integrations::provide_shell_auth_menu`].
#[component]
pub fn NotificationsLayout() -> impl IntoView {
    let app_name = AppMetadata::name().to_string();
    let selected_value = RwSignal::new(None::<String>);
    let open_categories = RwSignal::new(Vec::<String>::new());

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
            <ShellLeftNav slot>
                <Navigation config=NavigationConfig::new().with_selected_value(selected_value).with_open_categories(open_categories)>
                    <NavigationMaterial slot />
                    <NavigationBody slot>
                        <div id="nav-notifications-inbox">
                            <NavigationLink path=paths::ROOT value=paths::ROOT icon=icondata::AiInboxOutlined test_id="nav-notifications-inbox">"Inbox"</NavigationLink>
                        </div>
                    </NavigationBody>
                </Navigation>
            </ShellLeftNav>
            <Outlet />
        </UnifiedFieldShellLayout>
    }
}
