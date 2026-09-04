use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::Lazy;

/// Transparent component for Orbital component component preview catalog routes
///
/// Registry-driven previews are served at `/orbital/{slug}` via [`PreviewSlugPage`](crate::preview::PreviewSlugPage).
#[allow(missing_docs)]
#[orbital_macros::orbital_routes_extract]
#[component(transparent)]
pub fn OrbitalComponentRoutes() -> impl leptos_router::MatchNestedRoutes + Clone {
    use crate::lazy_routes::{ComponentPreviewRoute, IntroductionPageRoute, PreviewSlugPageRoute};
    use crate::pages::OrbitalDevLayout;
    use leptos_router::path;

    view! {
        <ParentRoute path=path!("orbital") view=OrbitalDevLayout>
            <Route path=path!("") view={Lazy::<IntroductionPageRoute>::new()} />
            <Route path=path!("components") view={Lazy::<IntroductionPageRoute>::new()} />
            <Route path=path!("shell") view={Lazy::<ComponentPreviewRoute>::new()} />
            <Route path=path!("/*slug") view={Lazy::<PreviewSlugPageRoute>::new()} />
        </ParentRoute>
    }
    .into_inner()
}
