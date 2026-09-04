use leptos::prelude::*;
use leptos_router::components::Outlet;
use uf_integrations::{
    HostAuthMenu, ShellAppBar, ShellAuthMenu, ShellLeftNav, UnifiedFieldAppBar,
    UnifiedFieldShellLayout,
};
use uf_product::components::{BackToTop, Container, Flex, FlexGap};

use crate::AppMetadata;

use super::catalog_nav::PreviewCatalogNav;
use super::catalog_search::PreviewCatalogSearch;

/// Full-page catalog shell using the shared Unified Field app bar + left nav.
///
/// Auth menu comes from the host via [`uf_integrations::provide_shell_auth_menu`].
#[component]
pub fn PreviewCatalogShell() -> impl IntoView {
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
            <ShellLeftNav slot>
                <PreviewCatalogNav />
            </ShellLeftNav>
            <div data-testid="preview-catalog-shell">
                <Container max_width="1200px".to_string()>
                    <Flex vertical=true gap=FlexGap::Medium>
                        <PreviewCatalogSearch />
                        <Outlet />
                    </Flex>
                    <BackToTop />
                </Container>
            </div>
        </UnifiedFieldShellLayout>
    }
}
