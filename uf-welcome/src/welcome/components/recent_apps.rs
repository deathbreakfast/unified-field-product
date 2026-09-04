//! Recent apps card (Spectra-backed).

use leptos::prelude::*;
use leptos::tachys::view::any_view::IntoAny;
use leptos_router::components::A;
use uf_product::primitives::{Button, ButtonAppearance};

use crate::welcome::server::get_recent_apps;

use super::app_links_body::AppLinksBody;
use super::WelcomeCard;

/// Recent apps card body.
#[component]
pub fn RecentAppsCardBody() -> impl IntoView {
    let links = Resource::new(|| (), |()| async move { get_recent_apps().await });
    view! {
        <div data-testid="recent-apps-card">
            <AppLinksBody
                links=links
                empty_message="Nothing yet. Visit an app to see it here."
                error_label="Failed to load recent apps"
            />
        </div>
    }
}

/// Recent apps card for the welcome page.
#[component]
pub fn RecentAppsCard() -> impl IntoView {
    view! {
        <div id="welcome-recent-apps-card">
            <WelcomeCard
                title="Recent apps"
                subtitle="Apps you opened recently"
                footer=move || view! {
                    <A href="/apps" attr:style="text-decoration: none;">
                        <Button appearance=ButtonAppearance::Secondary>"View all apps"</Button>
                    </A>
                }.into_any()
            >
                <RecentAppsCardBody />
            </WelcomeCard>
        </div>
    }
}
