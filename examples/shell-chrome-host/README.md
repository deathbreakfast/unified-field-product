# Shell chrome host

**Teaches:** `UnifiedFieldShellLayout`, `UnifiedFieldAppBar`, auth menu slot, default
app-bar utilities (`uf-help` / `uf-apps` / `uf-appearance`), `SearchSourcePicker`,
`UnifiedFieldComingSoonPage`, `UnifiedFieldNotFoundPage`, plus mounting
`UfAppsRoutes` / `UfWelcomeRoutes`.

**Topology:** Embedded (one Axum + Leptos process). Stub auth-menu label — no
Lepton session.

## Copy into your host

| File | What to take |
|------|----------------|
| [`Cargo.toml`](Cargo.toml) | `ssr` / `hydrate` feature graph and product crate deps (`uf-product`, `uf-integrations`, offerings, apps/welcome) |
| [`src/main.rs`](src/main.rs) | Axum router, `/pkg` + `/fonts` static mounts, `leptos_routes` boot |
| [`src/lib.rs`](src/lib.rs) | `orbital_shell` + `OrbitalTemplate`, shell layout slots, route mounts |

Workspace Leptos metadata for this package lives in the repo-root `Cargo.toml`
(`[[workspace.metadata.leptos]]` name `shell-chrome-host`). Point `/fonts` at your
Orbital `public/fonts` tree; replace the relative default in `src/main.rs` when
your Orbital checkout is elsewhere.

## SSR check (documented gate)

```bash
export CARGO_BUILD_JOBS=1
export CARGO_TARGET_DIR=target-uf-product
cargo check -p shell-chrome-host --features ssr
```

**Success:** package compiles under `ssr`.

## Hydrate / browser (optional)

Full client hydrate needs `cargo-leptos` and a `wasm32-unknown-unknown` target.
SSR check above is the documented success path; hydrate is optional for local teaching.

```bash
export LEPTOS_OUTPUT_NAME=shell-chrome-host
# `--split` matches uf-apps / uf-welcome lazy routes (enabled on this host).
cargo leptos watch --split --project shell-chrome-host
```

Open <http://127.0.0.1:3040/> — home shows the search picker; left nav links to coming-soon, 404, apps, and welcome.

**Look next:** [`../component-preview-host`](../component-preview-host/) for `/orbital` catalog hosting.
