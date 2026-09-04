//! Popular apps across the fleet (Spectra-backed).

use leptos::prelude::*;
use leptos::tachys::view::any_view::IntoAny;
use leptos_router::components::A;
use uf_product::primitives::{Button, ButtonAppearance};

use crate::welcome::server::get_popular_apps;

use super::app_links_body::AppLinksBody;
use super::WelcomeCard;

/// Popular apps card body.
#[component]
pub fn PopularAppsCardBody() -> impl IntoView {
    let links = Resource::new(|| (), |()| async move { get_popular_apps().await });
    view! {
        <div data-testid="popular-apps-card">
            <AppLinksBody
                links=links
                empty_message="No popular apps yet."
                error_label="Failed to load popular apps"
            />
        </div>
    }
}

/// Popular apps card for the welcome page.
#[component]
pub fn PopularAppsCard() -> impl IntoView {
    view! {
        <div id="welcome-popular-apps-card">
            <WelcomeCard
                title="Popular"
                subtitle="Most visited across this host"
                footer=move || view! {
                    <A href="/apps" attr:style="text-decoration: none;">
                        <Button appearance=ButtonAppearance::Secondary>"View all apps"</Button>
                    </A>
                }.into_any()
            >
                <PopularAppsCardBody />
            </WelcomeCard>
        </div>
    }
}
