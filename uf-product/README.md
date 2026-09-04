# uf-product

Unified Field product overlay for Leptos hosts: session bridge, route guards, app
registry, appearance preferences, and design-system re-exports used by
official uf-app UIs.

Task index and API tables:
`cargo doc -p uf-product --features ssr --open`.

## Design language

Orbital defines the accessible product vocabulary (spacing, typography, elevation,
materials). Use Orbital layout primitives (`Box`, `Stack`, `Flex`, `Grid`,
`AutoGrid`, `Container`) and surface styling (`Material`). Author guidance lives
in the component preview app (`/orbital`).

## Quick start

Provide context once, hydrate it, then read the profile (or match `AuthSession` for guest vs signed-in):

```rust,ignore
use leptos::prelude::*;
use uf_product::{
    init_auth_resource, provide_auth_context, use_auth_state, use_authenticated_user,
    routes::RequireAuthenticated, AuthSession,
};

#[component]
fn AppRoot() -> impl IntoView {
    let auth = provide_auth_context(Default::default());
    let _session = init_auth_resource(&auth);
    view! {
        <SessionChip />
        <ProtectedPage />
    }
}

#[component]
fn SessionChip() -> impl IntoView {
    let session = use_auth_state();
    view! {
        <span>{move || match session.get() {
            AuthSession::Anonymous(_) => "Guest",
            AuthSession::Authenticated(_) => "Signed in",
        }}</span>
    }
}

#[component]
fn ProtectedPage() -> impl IntoView {
    let user = use_authenticated_user();
    view! {
        <RequireAuthenticated>
            <p>{move || {
                user.get()
                    .and_then(|u| u.display_name.clone())
                    .unwrap_or_else(|| "you".to_string())
            }}</p>
        </RequireAuthenticated>
    }
}
```

## Examples

| Example | When to use | Command | Success |
|---------|-------------|---------|---------|
| [`uf_app_registration`](examples/uf_app_registration.rs) | Register with `uf_app!` | `cargo run -p uf-product --example uf_app_registration --features ssr` | `/sample-beacon` in path list |
| [`app_route_paths`](examples/app_route_paths.rs) | Bare route discovery | `cargo run -p uf-product --example app_route_paths --features ssr` | Prints registered paths |
| [`auth_shell_host`](examples/auth_shell_host.rs) | Axum session gate + two sample apps | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=target-uf-product cargo run -p uf-product --example auth_shell_host --features ssr` | Deny/allow + two apps |

Full ladder: [examples/README.md](examples/README.md). Shell chrome and `/orbital`
hosts: workspace [`examples/`](../examples/).

## Related

- Shell app bar / layout / search picker: [`uf-integrations`](../uf-integrations/)
- `uf_app!` + build-time route scan: [`uf-product-macros`](../uf-product-macros/), [`uf-codegen`](../uf-codegen/)
- Search DTOs / registry: [`uf-search-core`](../uf-search-core/)

---

*Visual language is inspired by contemporary design systems
([Fluent 2 design principles](https://fluent2.microsoft.design/design-principles),
[Material layout guidance](https://m3.material.io/foundations/layout/intro)).*
