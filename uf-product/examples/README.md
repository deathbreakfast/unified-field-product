# uf-product examples

Runnable teaching hosts for product shell composition. Inventory registrations are collected at link time; these examples do not start a full HTTP product server unless noted.

| Example | When to use | Command | Success | Look next |
|---------|-------------|---------|---------|-----------|
| [`uf_app_registration`](uf_app_registration.rs) | Register an app with `uf_app!` | `cargo run -p uf-product --example uf_app_registration --features ssr` | Stdout: `uf_app_registration: OK` including `/sample-beacon` | Mount routes in `shell-chrome-host` |
| [`app_route_paths`](app_route_paths.rs) | Bare route discovery via raw inventory | `cargo run -p uf-product --example app_route_paths --features ssr` | Prints registered paths including `/example` | Prefer `uf_app_registration` for product apps |
| [`auth_shell_host`](auth_shell_host.rs) | Axum session gate + two sample apps (raw inventory) | `CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=target-uf-product cargo run -p uf-product --example auth_shell_host --features ssr` | Stdout: `auth_shell_host: OK — /shell deny/allow + two registered apps` | Leptos `RequireAuthenticated` in product hosts; not the same as this Axum demo |

Nested UI hosts (workspace `examples/`): copy `Cargo.toml` + `main.rs` + `lib.rs`. Documented gate is SSR `cargo check`; browser hydrate via `cargo leptos` is optional (needs wasm + cargo-leptos).

| Host | When to use | Command | Success |
|------|-------------|---------|---------|
| [`shell-chrome-host`](../../examples/shell-chrome-host/) | All `uf-integrations` shell surfaces | `cargo check -p shell-chrome-host --features ssr` | Compiles |
| [`component-preview-host`](../../examples/component-preview-host/) | Mount `/orbital` catalog | `cargo check -p component-preview-host --features ssr` | Compiles; open `/orbital/demo-status-pill` when running |

Host cards: [`../../examples/README.md`](../../examples/README.md).

Short auth-gate sample: uf-product crate docs (Getting started).

`cargo doc -p uf-product --open`.
