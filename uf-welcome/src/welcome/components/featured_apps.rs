//! Featured apps card.

use leptos::prelude::*;
use leptos::tachys::view::any_view::IntoAny;
use leptos_router::components::A;
use uf_product::primitives::{Button, ButtonAppearance};

use crate::welcome::server::{can_manage_welcome_featured, get_featured_apps, AppLinkDto};

use super::app_links_body::{AppLinksBody, AppLinksSkeleton};
use super::WelcomeCard;

/// Featured apps card body.
#[component]
pub fn FeaturedAppsCardBody() -> impl IntoView {
    // Browser-only fetch: avoids SSR empty freeze and StyleRegistry races on this host.
    let links = RwSignal::new(Option::<Result<Vec<AppLinkDto>, ServerFnError>>::None);
    Effect::new(move |_| {
        if !cfg!(target_arch = "wasm32") {
            return;
        }
        leptos::task::spawn_local_scoped(async move {
            links.set(Some(get_featured_apps().await));
        });
    });
    let resource = Resource::new(
        move || links.get(),
        |current| async move {
            match current {
                Some(v) => v,
                None => Ok(Vec::new()),
            }
        },
    );

    view! {
        <div data-testid="featured-apps-card">
            {move || match links.get() {
                None => view! { <AppLinksSkeleton /> }.into_any(),
                Some(_) => view! {
                    <AppLinksBody
                        links=resource
                        empty_message="No featured apps yet."
                        error_label="Failed to load featured apps"
                    />
                }.into_any(),
            }}
        </div>
    }
}

/// Featured apps card for the welcome page.
#[component]
pub fn FeaturedAppsCard() -> impl IntoView {
    let can_manage = Resource::new(
        || (),
        |()| async move { can_manage_welcome_featured().await },
    );

    view! {
        <div id="welcome-featured-card">
            <WelcomeCard
                title="Featured"
                subtitle="Promoted apps for this host"
                footer=move || view! {
                    <span id="welcome-featured-view-all">
                        <A href="/apps" attr:style="text-decoration: none;">
                            <Button appearance=ButtonAppearance::Secondary>"View all apps"</Button>
                        </A>
                    </span>
                    {move || match can_manage.get() {
                        Some(Ok(true)) => view! {
                            <A href="/welcome/admin" attr:style="text-decoration: none; margin-left: 8px;">
                                <span data-testid="manage-featured-link">
                                    <Button appearance=ButtonAppearance::Subtle>
                                        "Manage featured"
                                    </Button>
                                </span>
                            </A>
                        }.into_any(),
                        _ => ().into_any(),
                    }}
                }.into_any()
            >
                <FeaturedAppsCardBody />
            </WelcomeCard>
        </div>
    }
}
