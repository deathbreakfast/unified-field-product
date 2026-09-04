//! Routes that mount product shell / apps / welcome / notifications for Playwright.

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::{Outlet, ParentRoute, Redirect, Route, Router, Routes};
use leptos_router::path;
use uf_apps::UfAppsRoutes;
use uf_integrations::{
    provide_shell_auth_menu, HostAuthMenu, ShellAppBar, ShellAuthMenu, ShellLeftNav,
    UnifiedFieldAppBar, UnifiedFieldShellLayout,
};
use uf_notifications::NotificationsRoutes;
use uf_product::components::{
    Navigation, NavigationBody, NavigationConfig, NavigationLink, NavigationMaterial,
};
use uf_product::{orbital_shell, OrbitalTemplate};
use uf_welcome::UfWelcomeRoutes;

use crate::gate_demos::E2eAuthProvider;
use crate::harness_auth_menu::HarnessAuthMenu;
use crate::pages::{
    AuthSignInPage, ComingSoonDemoPage, GateEmailPage, GatePermissionAllowPage, GatePermissionPage,
    HomePage, NotFoundDemoPage, ScrollChromePage, UtilitiesOverridePage, WorkspaceSearchHitPage,
};
use uf_product::telemetry::{PageViewTracker, UfAppRouteEntry};

/// Route table for e2e PageViewTracker (welcome + apps + shell gates).
static E2E_ROUTE_TABLE: &[UfAppRouteEntry] = &[
    UfAppRouteEntry {
        app_id: "welcome",
        app_name: "Welcome",
        route_prefix: "/welcome",
        brand_seed: "#1a6f94",
    },
    UfAppRouteEntry {
        app_id: "apps",
        app_name: "Apps",
        route_prefix: "/apps",
        brand_seed: "#2d6a4f",
    },
    UfAppRouteEntry {
        app_id: "notifications",
        app_name: "Notifications",
        route_prefix: "/notifications",
        brand_seed: "#5c4d7a",
    },
];

/// SSR document shell.
pub fn shell(options: LeptosOptions) -> impl IntoView {
    orbital_shell(options, || view! { <App/> })
}

/// Root app: consumer-wired product chrome + seeded auth context.
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    #[cfg(feature = "ssr")]
    {
        crate::wire_e2e_permissions();
        provide_context(crate::e2e_higgs_config());
    }
    provide_shell_auth_menu(|| view! { <HarnessAuthMenu /> });
    uf_help::ensure_linked();
    uf_appearance::ensure_linked();
    uf_apps::ensure_app_bar_linked();
    uf_apps::ensure_help_linked();
    uf_welcome::ensure_help_linked();
    uf_notifications::ensure_notification_bell_linked();
    uf_notifications::ensure_help_steps_linked();
    crate::help_steps::ensure_help_steps_linked();

    view! {
        <OrbitalTemplate>
            <E2eAuthProvider>
                <Router>
                    <PageViewTracker routes=E2E_ROUTE_TABLE surface="main_shell".to_string() />
                    <Routes fallback=|| view! { <NotFoundDemoPage /> }>
                        <ParentRoute path=path!("") view=ChromeShell>
                            <Route path=path!("") view=HomePage />
                            <Route path=path!("scroll-chrome") view=ScrollChromePage />
                            <Route path=path!("workspace-search-hit") view=WorkspaceSearchHitPage />
                            <Route path=path!("gate/email") view=GateEmailPage />
                            <Route path=path!("gate/permission") view=GatePermissionPage />
                            <Route path=path!("gate/permission-allow") view=GatePermissionAllowPage />
                        </ParentRoute>
                        <Route path=path!("utilities-override") view=UtilitiesOverridePage />
                        <Route path=path!("coming-soon") view=ComingSoonDemoPage />
                        <Route path=path!("404") view=NotFoundDemoPage />
                        <ParentRoute path=path!("auth") view=|| view! { <Outlet /> }>
                            <Route
                                path=path!("")
                                view=|| {
                                    view! { <Redirect path=uf_product::paths::AUTH_SIGNIN /> }
                                }
                            />
                            <Route path=path!("signin") view=AuthSignInPage />
                        </ParentRoute>
                        <UfAppsRoutes />
                        <UfWelcomeRoutes />
                        <NotificationsRoutes />
                    </Routes>
                </Router>
            </E2eAuthProvider>
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
                <UnifiedFieldAppBar app_name="Product UI e2e".to_string()>
                    <ShellAuthMenu slot:auth_menu>
                        <HostAuthMenu />
                    </ShellAuthMenu>
                </UnifiedFieldAppBar>
            </ShellAppBar>
            <ShellLeftNav slot>
                // Thin testid wrapper only — layout comes from Orbital Navigation.
                <div data-testid="shell-chrome-left-nav">
                    <Navigation config=NavigationConfig::new().with_selected_value(selected_value).with_open_categories(open_categories)>
                        <NavigationMaterial slot />
                        <NavigationBody slot>
                            <NavigationLink path="/" value="/" icon=icondata::AiHomeOutlined exact=true test_id="nav-home">"Home"</NavigationLink>
                            <NavigationLink path="/notifications" value="/notifications" icon=icondata::AiInboxOutlined test_id="nav-notifications-inbox">"Inbox"</NavigationLink>
                            <NavigationLink path="/coming-soon" value="/coming-soon" icon=icondata::AiClockCircleOutlined test_id="nav-coming-soon">"Coming soon"</NavigationLink>
                            <NavigationLink path="/404" value="/404" icon=icondata::AiFileUnknownOutlined test_id="nav-not-found">"Not found"</NavigationLink>
                            <NavigationLink path="/utilities-override" value="/utilities-override" icon=icondata::AiToolOutlined test_id="nav-utilities-override">"Utilities override"</NavigationLink>
                            <NavigationLink path="/apps" value="/apps" icon=icondata::AiAppstoreOutlined test_id="nav-apps">"Apps"</NavigationLink>
                            <NavigationLink path="/welcome" value="/welcome" icon=icondata::AiSmileOutlined test_id="nav-welcome">"Welcome"</NavigationLink>
                            <NavigationLink path="/gate/email" value="/gate/email" icon=icondata::AiMailOutlined test_id="nav-gate-email">"Email gate"</NavigationLink>
                            <NavigationLink path="/gate/permission" value="/gate/permission" icon=icondata::AiLockOutlined test_id="nav-gate-permission">"Permission gate"</NavigationLink>
                            <NavigationLink path="/gate/permission-allow" value="/gate/permission-allow" icon=icondata::AiSafetyCertificateOutlined test_id="nav-gate-permission-allow">"Permission allow"</NavigationLink>
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
