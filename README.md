# Unified Field Product

[![CI](https://github.com/unified-field-dev/unified-field-product/actions/workflows/ci.yml/badge.svg)](https://github.com/unified-field-dev/unified-field-product/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

[GitHub](https://github.com/unified-field-dev/unified-field-product) · `cargo doc -p uf-product --open`

Product shell APIs for Unified Field Leptos hosts. Hosts (for example
`lepton-uf-app`) depend on these crates for shell chrome, session/auth gates,
app registry, workspace search, appearance preferences, and design-system
primitives. This repository is not a standalone host binary.

```toml
[dependencies]
# Pin tag or rev — do not use branch = "main".
uf-product = { git = "https://github.com/unified-field-dev/unified-field-product", package = "uf-product", rev = "REPLACE_WITH_PIN" }
```

## What you get

- **App registration** — `uf_app!` in product apps; build-time route discovery via `uf-codegen`
- **Runtime registry and gates** — `AppRegistration` / `AppRegistry`, `RequireAuthenticated`, permission manifest shapes (`uf-product`)
- **Shell chrome** — app bar, shell layout, search picker, 404 / coming-soon pages (`uf-integrations`)
- **Search contracts** — client DTOs and SSR provider registry (`uf-search-core`)
- **Session and appearance** — host session bridge, light/dark/brand preferences, page-view telemetry (`uf-product`)
- **Design system** — Orbital primitives and components available through `uf-product` for app crates
- **Product apps** — signed-in welcome (`uf-welcome`), apps directory (`uf-apps`),
  component preview (`uf-component-preview`)
- **Notifications** — App-bar bell with live unread badge, dropdown preview, and
  auth-gated `/notifications` inbox (`uf-notifications`); enable
  `offering-notifications` (or `full`) so the shell picks up the inventory bell
- **Optional offerings** — Help and Appearance markers (`uf-help`, `uf-appearance`)

For types and task indexes, open the crate docs:
`cargo doc -p uf-product --features ssr --open` or
`cargo doc -p uf-integrations --features ssr --open`.

## Quick start

```rust,ignore
use leptos::prelude::*;
use uf_product::{use_authenticated_user, routes::RequireAuthenticated};

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

Product apps register with `uf_app!` and appear in the shell after codegen / inventory discovery.

## Examples

Nested Leptos hosts (copy `Cargo.toml` + `main.rs` + `lib.rs`): see [`examples/README.md`](examples/README.md).
Inventory / auth oneshots live under crate `examples/`.

| Host | When to use | Command | Success | Look next |
|------|-------------|---------|---------|-----------|
| [`uf_app_registration`](uf-product/examples/uf_app_registration.rs) | Register with `uf_app!` | `cargo run -p uf-product --example uf_app_registration --features ssr` | `/sample-beacon` in path list | `shell-chrome-host` |
| [`app_route_paths`](uf-product/examples/app_route_paths.rs) | Raw inventory route discovery | `cargo run -p uf-product --example app_route_paths --features ssr` | Prints registered paths | Prefer `uf_app_registration` |
| [`auth_shell_host`](uf-product/examples/auth_shell_host.rs) | Axum session gate + sample apps | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=target-uf-product cargo run -p uf-product --example auth_shell_host --features ssr` | Deny/allow + two apps | Leptos `RequireAuthenticated` (not this Axum demo) |
| [`query_many_stub`](uf-search-core/examples/query_many_stub.rs) | Search registry fan-out | `cargo run -p uf-search-core --example query_many_stub --features ssr` | `Beacon Alpha` hit | `SearchSourcePicker` in shell host |
| [`emit_routes_table`](uf-codegen/examples/emit_routes_table.rs) | Codegen route includes | `cargo run -p uf-codegen --example emit_routes_table` | Generated files mention `sample-beacon` | Host `build.rs` |
| [`shell-chrome-host`](examples/shell-chrome-host/) | All integrations shell surfaces | `cargo check -p shell-chrome-host --features ssr` | Compiles (SSR gate; hydrate via `cargo leptos` optional) | `component-preview-host` |
| [`component-preview-host`](examples/component-preview-host/) | Mount `/orbital` catalog | `cargo check -p component-preview-host --features ssr` | Compiles; slug `demo-status-pill` | Expose-component walkthrough in that README |

Full uf-product ladder: [`uf-product/examples/README.md`](uf-product/examples/README.md).
Nested host cards: [`examples/README.md`](examples/README.md).

## Verify

See [`docs/VERIFICATION.md`](docs/VERIFICATION.md) for fmt, clippy, tests, and rustdoc gates.
Workspace default allows broken intra-doc links; primary packages enforce denial only when
you set `RUSTDOCFLAGS` as documented there.

CI runs on every push and PR ([`.github/workflows/ci.yml`](.github/workflows/ci.yml)):
package fmt / clippy (`-D warnings`) / tests, plus primary `cargo doc` with
broken intra-doc link denial. No root `deny.toml` yet, so deny is not in CI.
Playwright e2e stays local.

```bash
export CARGO_BUILD_JOBS=1
export CARGO_TARGET_DIR=target-uf-product
cargo check --workspace
cargo run -p uf-product --example uf_app_registration --features ssr
```

## FAQ

**Is this a runnable host?** No. Depend on these crates from a Leptos host such as
`lepton-uf-app`. Teaching mounts live under `examples/` and `uf-product/examples/`.

**Which crate do I start with?** Zone apps and session/guards: `uf-product`. Shell
chrome: `uf-integrations`. App registration macros: `uf-product-macros` plus
`uf-codegen` in the host `build.rs`.

**How do I pin the dependency?** Use a git `rev` or tag. Avoid `branch = "main"`.

## License

MIT. See [LICENSE](LICENSE), [CONTRIBUTING.md](CONTRIBUTING.md), [SECURITY.md](SECURITY.md), and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
