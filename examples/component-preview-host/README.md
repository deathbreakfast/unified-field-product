# Component preview host

**Teaches:** Mounting `uf_component_preview::OrbitalComponentRoutes` so the catalog lives at `/orbital`, including left-nav entries from the preview registry.

**Topology:** Embedded (one Axum + Leptos process).

## Copy into your host

| File | What to take |
|------|----------------|
| [`Cargo.toml`](Cargo.toml) | `ssr` / `hydrate` features and `uf-component-preview` + `uf-product` deps |
| [`src/main.rs`](src/main.rs) | Axum + Leptos boot (`/pkg`, `leptos_routes`) |
| [`src/lib.rs`](src/lib.rs) | Redirect home → `/orbital`, mount `OrbitalComponentRoutes` |

Workspace Leptos metadata: repo-root `[[workspace.metadata.leptos]]` name
`component-preview-host` (site addr `127.0.0.1:3042`).

## SSR check (documented gate)

```bash
export CARGO_BUILD_JOBS=1
export CARGO_TARGET_DIR=target-uf-product
cargo check -p component-preview-host --features ssr
```

**Success:** package compiles under `ssr`; teaching slug `demo-status-pill` is registered in `uf-component-preview`.

## Hydrate / browser (optional)

Needs `cargo-leptos` and `wasm32-unknown-unknown`. Optional for release gates; SSR check is the success path.

```bash
export LEPTOS_OUTPUT_NAME=component-preview-host
cargo leptos watch --split --project component-preview-host
```

Open <http://127.0.0.1:3042/orbital> — catalog index. Teaching slug: `/orbital/demo-status-pill`.

## Expose your own component

1. Annotate the widget with `#[component_doc(..., preview_slug = "...")]` in the owning crate (enable that crate's `preview` feature).
2. Export registrations with `preview_registrations!` (see Orbital `orbital-macros` README).
3. Merge in [`uf-component-preview/src/preview/registry.rs`](../../uf-component-preview/src/preview/registry.rs) via `.extend(your_crate::preview::all())`.
4. Rebuild this host — the slug appears under left nav and at `/orbital/{slug}`.

In-repo teaching widget: `DemoStatusPill` → slug `demo-status-pill` under Examples / Unified Field.

Orbital authoring detail: [component-testing.md](https://github.com/unified-field-dev/orbital/blob/main/docs/component-testing.md).

**Look next:** [`../shell-chrome-host`](../shell-chrome-host/) for product shell chrome.
