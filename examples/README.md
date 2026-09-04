# Examples

Teaching hosts for product shell composition. Each card: when to use · command · success · look next.

Runnable Axum oneshots and inventory ladders live under crate `examples/` (`uf-product`,
`uf-search-core`, `uf-codegen`). Nested hosts here are the Leptos SSR mounts you copy into a
product binary.

## Canonical path

### `shell-chrome-host` — product shell chrome

**Teaches:** `UnifiedFieldShellLayout`, `UnifiedFieldAppBar`, auth-menu slot, default app-bar
utilities (`uf-help` / `uf-apps` / `uf-appearance`), `SearchSourcePicker`, coming-soon / 404
pages, plus mounting `UfAppsRoutes` / `UfWelcomeRoutes`.

**Copy:** `Cargo.toml` (ssr/hydrate feature graph), `src/main.rs` (Axum + Leptos boot),
`src/lib.rs` (shell composition). Point `/fonts` at your Orbital `public/fonts` tree
(the default `fonts_dir` in `main.rs` is a relative checkout path you will usually
replace).

```bash
export CARGO_BUILD_JOBS=1
export CARGO_TARGET_DIR=target-uf-product
cargo check -p shell-chrome-host --features ssr
```

**Success:** package compiles under `ssr`.

**Hydrate / browser:** optional. Needs `cargo-leptos`, a `wasm32` target, and
`cargo leptos watch --split --project shell-chrome-host`. The documented SSR gate
is `cargo check` above; hydrate is optional for local teaching.

**Next step:** [`component-preview-host`](component-preview-host/) for `/orbital`, or mount the
same shell slots in `lepton-uf-app`.

### `component-preview-host` — `/orbital` catalog

**Teaches:** Mounting `uf_component_preview::OrbitalComponentRoutes` at `/orbital`, including
left-nav entries from the preview registry.

**Copy:** same three files as above; swap shell chrome for `OrbitalComponentRoutes` in `lib.rs`.

```bash
export CARGO_BUILD_JOBS=1
export CARGO_TARGET_DIR=target-uf-product
cargo check -p component-preview-host --features ssr
```

**Success:** package compiles; teaching slug `demo-status-pill` is registered in the preview crate.

**Hydrate / browser:** optional (`cargo leptos watch --split --project component-preview-host`,
port 3042). Same gate rule as shell chrome — SSR check is the documented success path.

**Next step:** Expose-component walkthrough in that host README; full product host =
`lepton-uf-app`.

| Host | When to use | Command | Success | Look next |
|------|-------------|---------|---------|-----------|
| [`shell-chrome-host`](shell-chrome-host/) | All integrations shell surfaces | `cargo check -p shell-chrome-host --features ssr` | Compiles | `component-preview-host` / `lepton-uf-app` |
| [`component-preview-host`](component-preview-host/) | Mount `/orbital` catalog | `cargo check -p component-preview-host --features ssr` | Compiles; slug `demo-status-pill` | Expose-component walkthrough |

Inventory / auth oneshots (not nested hosts): [`../uf-product/examples/README.md`](../uf-product/examples/README.md).
