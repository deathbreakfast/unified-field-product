//! Product-shell Coming Soon page — same app bar chrome as the rest of Unified Field.

use leptos::prelude::*;
use leptos_router::hooks::{use_location, use_navigate};
use uf_product::components::{ComingSoon, EmptyState, EmptyStateCallToAction};
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

/// Unit progress (`0.0..=1.0`) for the Coming Soon pill, keyed by gated app path prefix.
pub fn coming_soon_fill_for_path(pathname: &str) -> f64 {
    let path = if pathname.len() > 1 {
        pathname.trim_end_matches('/')
    } else {
        pathname
    };
    if path.starts_with("/chronon") {
        0.90
    } else if path.starts_with("/valence") {
        0.70
    } else if path.starts_with("/spectra") {
        0.30
    } else if path.starts_with("/boson") {
        0.90
    } else if path.starts_with("/permission") || path.starts_with("/secrets") {
        0.50
    } else if path.starts_with("/photon") {
        0.70
    } else {
        0.90
    }
}

/// Coming soon page composed with [`UnifiedFieldShellLayout`] and [`UnifiedFieldAppBar`].
///
/// Pass [`ShellAuthMenu`] so Sign In / account match every other product shell.
#[component]
pub fn UnifiedFieldComingSoonPage(
    /// Auth menu slot — host passes `lepton_shell::AppBarUserMenu` here.
    auth_menu: ShellAuthMenu,
) -> impl IntoView {
    let navigate = use_navigate();
    let location = use_location();

    view! {
        <div data-testid="unified-field-coming-soon-page">
            <UnifiedFieldShellLayout>
                <ShellAppBar slot>
                    <UnifiedFieldAppBar
                        app_name="Unified Field".to_string()
                        app_id="unified-field"
                        homepage_url="/".to_string()
                        breadcrumbs=vec![BreadcrumbLink::new("Coming soon", "#")]
                        auth_menu=auth_menu
                    />
                </ShellAppBar>
                <div style=CENTER_STYLE>
                    <div style=CONTENT_STYLE>
                        <EmptyState
                            message="Coming soon"
                            description="This part of Unified Field is not available yet."
                        >
                            <EmptyStateCallToAction slot:call_to_action>
                                <div style="display: flex; flex-direction: column; align-items: center; gap: 16px; width: min(100%, 28rem);">
                                    {move || {
                                        let fill_to = coming_soon_fill_for_path(&location.pathname.get());
                                        view! {
                                            <ComingSoon
                                                brand_color="var(--colorBrandBackground, var(--orb-color-brand-background, #0f6cbd))".to_string()
                                                fill_to=fill_to
                                            />
                                        }
                                    }}
                                    <Button
                                        appearance=ButtonAppearance::Primary
                                        on_click=Callback::new(move |_| {
                                            navigate_back_or("/", &navigate);
                                        })
                                    >
                                        "Go back"
                                    </Button>
                                </div>
                            </EmptyStateCallToAction>
                        </EmptyState>
                    </div>
                </div>
            </UnifiedFieldShellLayout>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::coming_soon_fill_for_path;

    #[test]
    fn fill_targets_by_path() {
        assert!((coming_soon_fill_for_path("/chronon") - 0.90).abs() < f64::EPSILON);
        assert!((coming_soon_fill_for_path("/valence/schemas") - 0.70).abs() < f64::EPSILON);
        assert!((coming_soon_fill_for_path("/spectra") - 0.30).abs() < f64::EPSILON);
        assert!((coming_soon_fill_for_path("/boson/") - 0.90).abs() < f64::EPSILON);
        assert!((coming_soon_fill_for_path("/permission") - 0.50).abs() < f64::EPSILON);
        assert!((coming_soon_fill_for_path("/secrets") - 0.50).abs() < f64::EPSILON);
        assert!((coming_soon_fill_for_path("/photon") - 0.70).abs() < f64::EPSILON);
        assert!((coming_soon_fill_for_path("/unknown") - 0.90).abs() < f64::EPSILON);
    }
}
