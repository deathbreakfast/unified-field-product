# uf-product-macros

Proc macros for Unified Field product apps: app registration, server-fn context,
permission manifests, and search-source declarations.

Field tables and expand examples: `cargo doc -p uf-product-macros --open`.

Design-system macros (`#[component_doc]`, route extraction) live in `orbital-macros`.

## Features

- `uf_app!` — register a product app for shell discovery and `uf-codegen` scans
- `#[uf_product_macros::server]` — Leptos `#[server]` plus operation context / optional permission gate
- `#[derive(UfPermissionManifest)]` — crate-local permission manifest
- `define_search_sources!` — search source ids and SSR descriptors for `uf-search-core`

## Usage

```rust,ignore
use uf_product_macros::uf_app;

uf_app! {
    name: "Sample Beacon",
    id: "sample-beacon",
    description: "Teaching app registered with uf_app!",
    icon: "Cube",
    version: "0.1.0",
    routes: SampleBeaconRoutes,
    route_path: "/sample-beacon",
    // Optional: enables GitHub / docs.rs buttons on the Apps detail page
    // repository: "https://github.com/org/repo",
    // crate_name: "my-crate",
}
```

Runnable discovery: `cargo run -p uf-product --example uf_app_registration --features ssr`.

## Verify

```bash
cargo test -p uf-product-macros
cargo doc -p uf-product-macros --no-deps
```

## Related

- Build-time route discovery: [`uf-codegen`](../uf-codegen/)
- Runtime registry / guards: [`uf-product`](../uf-product/)
- Search registry: [`uf-search-core`](../uf-search-core/)
- Flat example: [`uf-product/examples/uf_app_registration.rs`](../uf-product/examples/uf_app_registration.rs)
