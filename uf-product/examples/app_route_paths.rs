//! Discover the route prefixes registered by product apps (raw `inventory::submit!`).
//!
//! Prefer [`uf_app_registration`](uf_app_registration.rs) when teaching registered product apps —
//! that example uses `uf_app!`, which is what product crates ship.
//!
//! ## Command
//! ```bash
//! cargo run -p uf-product --example app_route_paths --features ssr
//! ```
//!
//! ## Success
//! Prints `Registered route paths: […]` including `/example`.
//!
//! ## Look next
//! `uf_app_registration` for the macro path; `auth_shell_host` for an Axum session gate.

use uf_product::routes::get_all_app_route_paths;
use uf_product::{inventory, AppRegistration};

inventory::submit! {
    AppRegistration {
        id: "example",
        name: "Route discovery example",
        description: "A minimal app registration for route discovery.",
        icon: "Route",
        route_path: "/example",
        repository: None,
        crate_name: None,
        brand_seed: None,
        permission_manifest: None,
    }
}

fn main() {
    let paths = get_all_app_route_paths();

    assert!(paths.contains(&"/example"));
    println!("Registered route paths: {paths:?}");
}
