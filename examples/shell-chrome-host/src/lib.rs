//! Shell chrome teaching host library entry.
//!
//! Copy [`App`] / [`shell`] composition into a product host: `OrbitalTemplate`,
//! `UnifiedFieldShellLayout` slots, then mount zone routes (`UfAppsRoutes`,
//! `UfWelcomeRoutes`, …).
#![allow(missing_docs)]

mod pages;

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::{Outlet, ParentRoute, Route, Router, Routes};
use leptos_router::path;
use uf_apps::UfAppsRoutes;
use uf_integrations::{
    provide_shell_auth_menu, HostAuthMenu, ShellAppBar, ShellAuthMenu, ShellLeftNav,
    UnifiedFieldAppBar, UnifiedFieldShellLayout,
};
use uf_product::components::{
    Navigation, NavigationBody, NavigationConfig, NavigationLink, NavigationMaterial,
};
use uf_product::{orbital_shell, OrbitalTemplate};
use uf_welcome::UfWelcomeRoutes;

use pages::{ComingSoonDemoPage, HomePage, NotFoundDemoPage};

/// SSR document shell.
pub fn shell(options: LeptosOptions) -> impl IntoView {
    orbital_shell(options, || view! { <App/> })
}

/// Root app: product shell chrome + sample app mounts.
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    provide_shell_auth_menu(|| view! { <span data-testid="demo-auth-menu">"Demo user"</span> });
    // Link offering inventory (help/appearance via uf-integrations `full`; apps via this dep).
    // `offering-notifications` is also in `full`; depend on `uf-notifications` in hosts that
    // want HostNotificationBell filled via inventory (this chrome host leaves the slot empty).
    uf_help::ensure_linked();
    uf_appearance::ensure_linked();
    uf_apps::ensure_app_bar_linked();

    view! {
        <OrbitalTemplate>
            <Router>
                <Routes fallback=|| view! { <p>"Not found"</p> }>
                    <ParentRoute path=path!("") view=ChromeShell>
                        <Route path=path!("") view=HomePage />
                    </ParentRoute>
                    <Route path=path!("coming-soon") view=ComingSoonDemoPage />
                    <Route path=path!("404") view=NotFoundDemoPage />
                    <UfAppsRoutes />
                    <UfWelcomeRoutes />
                </Routes>
            </Router>
        </OrbitalTemplate>
    }
}

#[component]
fn ChromeShell() -> impl IntoView {
    let selected_value = RwSignal::new(None::<String>);
    let open_categories = RwSignal::new(Vec::<String>::new());

    view! {
        <UnifiedFieldShellLayout>
            <ShellAppBar slot>
                <UnifiedFieldAppBar app_name="Shell chrome host".to_string()>
                    <ShellAuthMenu slot:auth_menu>
                        <HostAuthMenu />
                    </ShellAuthMenu>
                </UnifiedFieldAppBar>
            </ShellAppBar>
            <ShellLeftNav slot>
                <div data-testid="shell-chrome-left-nav">
                    <Navigation config=NavigationConfig::new().with_selected_value(selected_value).with_open_categories(open_categories)>
                        <NavigationMaterial slot />
                        <NavigationBody slot>
                            <NavigationLink path="/" value="/" icon=icondata::AiHomeOutlined exact=true test_id="nav-home">"Home"</NavigationLink>
                            <NavigationLink path="/coming-soon" value="/coming-soon" icon=icondata::AiClockCircleOutlined test_id="nav-coming-soon">"Coming soon"</NavigationLink>
                            <NavigationLink path="/404" value="/404" icon=icondata::AiFileUnknownOutlined test_id="nav-not-found">"Not found"</NavigationLink>
                            <NavigationLink path="/apps" value="/apps" icon=icondata::AiAppstoreOutlined test_id="nav-apps">"Apps"</NavigationLink>
                            <NavigationLink path="/welcome" value="/welcome" icon=icondata::AiSmileOutlined test_id="nav-welcome">"Welcome"</NavigationLink>
                        </NavigationBody>
                    </Navigation>
                </div>
            </ShellLeftNav>
            <Outlet />
        </UnifiedFieldShellLayout>
    }
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
    uf_product::hide_boot_loader();
}
