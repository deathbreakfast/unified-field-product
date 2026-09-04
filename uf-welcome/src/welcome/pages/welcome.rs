//! The signed-in welcome page: featured and Spectra usage cards.

use crate::welcome::components::{
    FeaturedAppsCard, MyMostUsedCard, PopularAppsCard, RecentAppsCard,
};
use leptos::prelude::*;
use turf::inline_style_sheet_values;
use uf_product::components::ContentContainer;

/// The signed-in welcome landing page.
#[component]
pub fn WelcomePage() -> impl IntoView {
    let _ = crate::help_steps::ensure_help_steps_linked();
    // Turf grid kept intentionally: AutoGrid + many Orbital inject_style cards panic the
    // e2e SSR StyleRegistry owner on this host (disposed reactive). Match prior welcome shell.
    let (style_sheet, class_names) = inline_style_sheet_values! {
        .WelcomeGrid {
            display: grid;
            grid-template-columns: repeat(2, 1fr);
            gap: 16px;
        }

        @media (max-width: 879px) {
            .WelcomeGrid {
                grid-template-columns: 1fr;
            }
        }
    };

    view! {
        <style>{style_sheet}</style>
        <ContentContainer data_testid="welcome-page">
            <div class=class_names.welcome_grid>
                <FeaturedAppsCard />
                <RecentAppsCard />
                <MyMostUsedCard />
                <PopularAppsCard />
            </div>
        </ContentContainer>
    }
}
