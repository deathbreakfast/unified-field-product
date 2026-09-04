# uf-apps

Apps directory for Unified Field product hosts. Lists every app registered with
`uf_app!` on a searchable `/apps` index, and opens a detail page with name,
description, primary route, and optional repository / docs.rs links.

Routes and types: `cargo doc -p uf-apps --open`.

## Features

- **Index** — searchable grid (`AppsIndexPage` / `get_apps_page`)
- **Detail** — overview card (`AppDetailPage` / `get_app_overview`)
- **App-bar launcher** — `ensure_app_bar_linked` + `AppBarAppsButton` /
  `AppsLauncher` (centered dialog typeahead; select navigates to `route_path`)
- **Self-registration** — this crate is the `"apps"` app at `/apps`

Hosts that use the default product app bar should call `ensure_app_bar_linked()`
so the Apps control shows up. Routes work without it; the launcher button does not.

## Define an app

```rust
use uf_product_macros::uf_app;

uf_app! {
    name: "Sample Beacon",
    id: "sample-beacon",
    description: "Teaching app registered with uf_app!",
    icon: "Cube",
    version: "0.1.0",
    routes: SampleBeaconRoutes,
    route_path: "/sample-beacon",
}
```

Inventory smoke:

```bash
cargo run -p uf-product --example uf_app_registration --features ssr
```

## Mount

```rust,ignore
use leptos_router::components::Routes;
use uf_apps::{ensure_app_bar_linked, UfAppsRoutes};

ensure_app_bar_linked();

view! {
    <Routes fallback=|| "not found">
        <UfAppsRoutes />
    </Routes>
}
```

Routes:

- `/apps` — index
- `/apps/:app_name` — detail

Teaching host: [`examples/shell-chrome-host`](../examples/shell-chrome-host/).

## Verify

```bash
cargo check -p uf-apps --features ssr
cargo check -p shell-chrome-host --features ssr
cargo doc -p uf-apps --no-deps
```

## Related

- Product overlay / registry: [`uf-product`](../uf-product/)
- Build-time `uf_app!` discovery: [`uf-codegen`](../uf-codegen/)
- Signed-in welcome landing: [`uf-welcome`](../uf-welcome/)
