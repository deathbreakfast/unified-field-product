//! Emit route include files from a tiny fixture workspace that uses `uf_app!`.
//!
//! ## When to use
//! See what host `build.rs` gets from `generate_registered_routes` without a full product host.
//!
//! ## Command
//! ```bash
//! cargo run -p uf-codegen --example emit_routes_table
//! ```
//!
//! ## Success
//! Stdout prints `emit_routes_table: OK` and the generated file contains `sample-beacon`.
//!
//! ## Look next
//! Host `build.rs` in a product app crate; runtime discovery: `uf-product` `uf_app_registration`.

#![allow(clippy::print_stdout, clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

use uf_codegen::{generate_registered_routes, RoutesCodegenConfig};

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixtures_root = manifest_dir.join("examples/fixtures");
    let out_dir =
        std::env::temp_dir().join(format!("uf-codegen-emit-routes-{}", std::process::id()));
    std::fs::create_dir_all(&out_dir).expect("temp out_dir");

    generate_registered_routes(&RoutesCodegenConfig {
        workspace_root: fixtures_root,
        out_dir: out_dir.clone(),
        extra_packages: vec![],
        excluded_packages: vec![],
    })
    .expect("generate_registered_routes");

    let routes = std::fs::read_to_string(out_dir.join("uf_registered_routes.rs"))
        .expect("read uf_registered_routes.rs");
    let table = std::fs::read_to_string(out_dir.join("uf_app_route_table.rs"))
        .expect("read uf_app_route_table.rs");

    assert!(
        routes.contains("sample-beacon") || table.contains("sample-beacon"),
        "expected sample-beacon in generated outputs\nroutes:\n{routes}\ntable:\n{table}"
    );
    println!(
        "emit_routes_table: OK — wrote {} and {}",
        out_dir.join("uf_registered_routes.rs").display(),
        out_dir.join("uf_app_route_table.rs").display()
    );
}
