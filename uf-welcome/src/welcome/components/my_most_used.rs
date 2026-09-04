//! My most-used apps card (Spectra-backed).

use leptos::prelude::*;
use leptos::tachys::view::any_view::IntoAny;
use leptos_router::components::A;
use uf_product::primitives::{Button, ButtonAppearance};

use crate::welcome::server::get_my_most_used;

use super::app_links_body::AppLinksBody;
use super::WelcomeCard;

/// My most used card body.
#[component]
pub fn MyMostUsedCardBody() -> impl IntoView {
    let links = Resource::new(|| (), |()| async move { get_my_most_used().await });
    view! {
        <div data-testid="my-most-used-card">
            <AppLinksBody
                links=links
                empty_message="No usage yet. Open a few apps and check back."
                error_label="Failed to load most used apps"
            />
        </div>
    }
}

/// My most used card for the welcome page.
#[component]
pub fn MyMostUsedCard() -> impl IntoView {
    view! {
        <div id="welcome-most-used-card">
            <WelcomeCard
                title="My most used"
                subtitle="Based on your visits"
                footer=move || view! {
                    <A href="/apps" attr:style="text-decoration: none;">
                        <Button appearance=ButtonAppearance::Secondary>"View all apps"</Button>
                    </A>
                }.into_any()
            >
                <MyMostUsedCardBody />
            </WelcomeCard>
        </div>
    }
}
