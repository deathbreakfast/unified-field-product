//! Register a product app with `uf_app!` and prove inventory discovery.
//!
//! ## When to use
//! Prefer this over raw `inventory::submit!` — product apps use the macro.
//!
//! ## Command
//! ```bash
//! cargo run -p uf-product --example uf_app_registration --features ssr
//! ```
//!
//! ## Success
//! Stdout prints `uf_app_registration: OK` with `/sample-beacon` in the path list.
//!
//! ## Look next
//! Mount routes in a host (`shell-chrome-host`); flat inventory smoke: `app_route_paths`.

#![allow(
    missing_docs,
    clippy::print_stdout,
    clippy::unwrap_used,
    clippy::expect_used
)]

use leptos::prelude::*;
use leptos_router::components::{ParentRoute, Route};
use leptos_router::path;
use uf_product::routes::get_all_app_route_paths;
use uf_product_macros::uf_app;

/// Minimal nested routes for the Sample Beacon teaching app.
#[component(transparent)]
fn SampleBeaconRoutes() -> impl leptos_router::MatchNestedRoutes + Clone {
    view! {
        <ParentRoute path=path!("sample-beacon") view=|| view! { <p>"Sample Beacon"</p> }>
            <Route path=path!("") view=|| view! { <p>"ok"</p> } />
        </ParentRoute>
    }
    .into_inner()
}

uf_app! {
    name: "Sample Beacon",
    id: "sample-beacon",
    description: "Teaching app registered with uf_app!",
    icon: "Cube",
    version: "0.1.0",
    routes: SampleBeaconRoutes,
    route_path: "/sample-beacon",
}

fn main() {
    let paths = get_all_app_route_paths();
    assert!(
        paths.iter().any(|p| *p == "/sample-beacon"),
        "expected /sample-beacon in {paths:?}"
    );
    println!("uf_app_registration: OK — paths={paths:?}");
}
