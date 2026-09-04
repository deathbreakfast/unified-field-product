//! Component preview teaching host — mounts `OrbitalComponentRoutes` at `/orbital`.
//!
//! Copy [`App`] / [`shell`] into a product host when you want the Orbital catalog
//! under `/orbital` (redirect home → catalog index).
#![allow(missing_docs)]

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::{Redirect, Route, Router, Routes};
use leptos_router::path;
use uf_component_preview::OrbitalComponentRoutes;
use uf_product::{orbital_shell, OrbitalTemplate};

/// SSR document shell.
pub fn shell(options: LeptosOptions) -> impl IntoView {
    orbital_shell(options, || view! { <App/> })
}

/// Root app: redirect home to `/orbital`, mount the catalog routes.
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <OrbitalTemplate>
            <Router>
                <Routes fallback=|| view! { <p>"Not found"</p> }>
                    <Route path=path!("") view=|| view! { <Redirect path="/orbital" /> } />
                    <OrbitalComponentRoutes />
                </Routes>
            </Router>
        </OrbitalTemplate>
    }
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
    uf_product::hide_boot_loader();
}
