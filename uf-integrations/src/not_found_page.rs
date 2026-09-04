//! Product-shell 404 page — same app bar chrome as the rest of Unified Field.

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use uf_product::components::{EmptyState, EmptyStateCallToAction, EMPTYSTATE_SAD_DOG_ILLUSTRATION};
use uf_product::nav::navigate_back_or;
use uf_product::primitives::{Button, ButtonAppearance};

use crate::{
    BreadcrumbLink, ShellAppBar, ShellAuthMenu, UnifiedFieldAppBar, UnifiedFieldShellLayout,
};

/// Fill the shell main column and center the empty state on both axes.
const CENTER_STYLE: &str = concat!(
    "min-height: calc(100dvh - var(--orbital-layout-header-inset, 48px)",
    " - 2 * var(--orb-space-block-lg, 16px));",
    "width: 100%;",
    "box-sizing: border-box;",
    "display: grid;",
    "place-items: center;",
);

const CONTENT_STYLE: &str = "max-width: 720px; width: max-content; justify-self: center;";

/// 404 page composed with [`UnifiedFieldShellLayout`] and [`UnifiedFieldAppBar`].
///
/// Pass [`ShellAuthMenu`] so Sign In / account match every other product shell.
#[component]
pub fn UnifiedFieldNotFoundPage(
    /// Auth menu slot — host passes `lepton_shell::AppBarUserMenu` here.
    auth_menu: ShellAuthMenu,
) -> impl IntoView {
    let navigate = use_navigate();
    view! {
        <div data-testid="unified-field-not-found-page">
            <UnifiedFieldShellLayout>
                <ShellAppBar slot>
                    <UnifiedFieldAppBar
                        app_name="Unified Field".to_string()
                        app_id="unified-field"
                        homepage_url="/".to_string()
                        breadcrumbs=vec![BreadcrumbLink::new("Not found", "#")]
                        auth_menu=auth_menu
                    />
                </ShellAppBar>
                <div style=CENTER_STYLE>
                    <div style=CONTENT_STYLE>
                        <EmptyState
                            message="Page not found"
                            description="The page you requested does not exist or may have moved."
                            illustration_src=EMPTYSTATE_SAD_DOG_ILLUSTRATION
                            illustration_alt="Sad dog illustration"
                        >
                            <EmptyStateCallToAction slot:call_to_action>
                                <Button
                                    appearance=ButtonAppearance::Primary
                                    on_click=Callback::new(move |_| {
                                        navigate_back_or("/", &navigate);
                                    })
                                >
                                    "Go back"
                                </Button>
                            </EmptyStateCallToAction>
                        </EmptyState>
                    </div>
                </div>
            </UnifiedFieldShellLayout>
        </div>
    }
}
