//! Host utilities slot override (empty children = no default pack).
#![allow(missing_docs)]

use leptos::prelude::*;
use uf_integrations::{
    provide_shell_auth_menu, AppBarUtilities, HostAuthMenu, ShellAppBar, ShellAuthMenu,
    UnifiedFieldAppBar, UnifiedFieldShellLayout,
};
use uf_product::components::{Body1, Title3};
use uf_product::primitives::{Flex, FlexGap};

#[component]
pub fn UtilitiesOverridePage() -> impl IntoView {
    provide_shell_auth_menu(|| view! { <span data-testid="demo-auth-menu">"Demo user"</span> });

    view! {
        <UnifiedFieldShellLayout>
            <ShellAppBar slot>
                <UnifiedFieldAppBar app_name="Utilities override".to_string()>
                    <AppBarUtilities slot:app_bar_utilities>
                        <span data-testid="custom-utilities-marker">"custom-only"</span>
                    </AppBarUtilities>
                    <ShellAuthMenu slot:auth_menu>
                        <HostAuthMenu />
                    </ShellAuthMenu>
                </UnifiedFieldAppBar>
            </ShellAppBar>
            <main data-testid="utilities-override-page" style="padding: 24px; max-width: 720px;">
                <Flex vertical=true gap=FlexGap::Medium full_width=true>
                    <Title3>"Utilities override"</Title3>
                    <Body1>"Host children replace the default offering pack."</Body1>
                </Flex>
            </main>
        </UnifiedFieldShellLayout>
    }
}
