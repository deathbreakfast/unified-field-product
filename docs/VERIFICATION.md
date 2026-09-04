# unified-field-product verification

Re-run after code or doc changes to confirm the product wiring crates still build
and document cleanly.

## Environment

```bash
export CARGO_BUILD_JOBS=1
export CARGO_TARGET_DIR=target-uf-product
```

This workspace pins `rust-toolchain.toml` to `nightly` (Leptos `nightly` features + Orbital). Use that channel for the commands below.

Valence / Orbital / Higgs / Photon resolve from `unified-field-dev` git `main` (see workspace `Cargo.toml`). CI shallow-clones those siblings into the operator layout when jobs need them.

## Layer 1 — Unit + integration

Preferred contract crates (macros + codegen + apps directory; avoid full UI graph when possible).
CI `test` also runs `uf-product` and `uf-welcome` with `ssr`:

```bash
cargo fmt -p uf-product-macros -p uf-codegen -p uf-apps -p uf-search-core -- --check
cargo clippy -p uf-product-macros -p uf-codegen -p uf-search-core --all-targets -- -D warnings
cargo test -p uf-product-macros -p uf-codegen
cargo test -p uf-search-core --features ssr
cargo test -p uf-apps --features ssr
cargo test -p uf-product --features ssr
cargo test -p uf-welcome --features ssr
cargo clippy -p uf-apps --features ssr --all-targets -- -D warnings
```

Step-up (`StepUpMode` parse + stable `STEP_UP:` prefixes; macro expansion):

```bash
cargo test -p uf-product-macros server_step_up
cargo test -p uf-product --lib step_up_mode_ -- --nocapture
cargo test -p uf-product --lib step_up_error_prefixes -- --nocapture
```

Additional SSR checks (not in the default CI `test` job):

```bash
cargo check -p uf-help -p uf-appearance --features ssr
cargo check -p uf-integrations --features ssr
cargo check -p uf-integrations --no-default-features --features ssr
cargo check -p uf-notifications --features ssr
```

Optional SSR helpers in `uf-product` (heavier graph):

```bash
cargo run -p uf-product --example uf_app_registration --features ssr
cargo run -p uf-product --example app_route_paths --features ssr
```

Search / codegen teaching examples:

```bash
cargo run -p uf-search-core --example query_many_stub --features ssr
cargo run -p uf-codegen --example emit_routes_table
```

Preview catalog + nested teaching hosts (SSR check is the gate; hydrate optional):

```bash
cargo check -p uf-component-preview --features ssr
cargo test -p uf-component-preview --features ssr --lib
cargo check -p shell-chrome-host --features ssr
cargo check -p component-preview-host --features ssr
```

Host copy cards: [`../examples/README.md`](../examples/README.md).

### leptos-lints (CI job `leptos-lints`)

Needs `cargo-dylint` / `dylint-link` 6.0.1 and toolchain `nightly-2025-05-14`
(see `.github/workflows/ci.yml`). Hydrate UI crates (`--no-deps`):

```bash
# cargo install cargo-dylint --locked --version 6.0.1
# cargo install dylint-link --locked --version 6.0.1
# rustup toolchain install nightly-2025-05-14 --component rustc-dev,llvm-tools-preview
export CARGO_RESOLVER_INCOMPATIBLE_RUST_VERSIONS=fallback
export RUSTFLAGS="-D warnings -Zcrate-attr=feature(stdarch_x86_avx512)"
cargo dylint --all -p uf-apps --no-deps -- --features hydrate
cargo dylint --all -p uf-welcome --no-deps -- --features hydrate
cargo dylint --all -p uf-integrations --no-deps -- --features hydrate
```

## Layer 2 — UI e2e

Product operator surfaces (shell, app-bar utilities, search, coming-soon/404, auth
gates, `/apps`, `/welcome`) plus **notifications** inbox/bell/Photon WS
(`end2end/tests/notifications.spec.ts`). Scenario IDs live in
[`uf-product-ui-e2e/README.md`](../uf-product-ui-e2e/README.md).
CI job `e2e` runs the same commands.

```bash
cd uf-product-ui-e2e/end2end && npm ci && npx playwright install chromium
cd ../..
cargo leptos end-to-end --project uf-product-ui-e2e
```

### Runtime scope (what this host boots)

`uf-product-ui-e2e` is a library-owned Axum + Leptos host with **mem Valence**, **mem Spectra**, a **session stub** for auth gates, and **in-process Photon** for `/ws/notifications` (notifications suite only).

| Runtime | Product feature need? | This host |
|---------|------------------------|-----------|
| Valence | Yes | Mem / harness Sqlite for notifications seed |
| Spectra | Yes (usage / page views) | Mem |
| Higgs | Yes on production hosts | Stubbed (session + harness Valence) |
| Gauge (`PermissionBackend`) | Yes for real permission allow | Harness backend + `permission_allow` session flag (`e2e.permission.allow` / deny) |
| Photon | Notifications unread WS only | In-process `/ws/notifications` |
| Chronon / Boson | No | Not installed |

Chronon / Boson (and broader Photon topics) still belong in IsolatedLab host e2e when testing host composition. Real Gauge `actor_can` belongs with gauge / site hosts that call `wire_gauge_permissions()`. This host’s harness `PermissionBackend` covers gate allow/deny UI only.

## Documentation gates

Workspace default: `broken_intra_doc_links` is **allow** (see root `Cargo.toml`).
After public API or integrator-doc changes, rebuild the **primary** packages with
broken-link **denial**:

| Package | Features | Link deny? |
|---------|----------|------------|
| `uf-product-macros`, `uf-codegen` | (none) | yes (`RUSTDOCFLAGS`) |
| `uf-search-core`, `uf-product`, `uf-integrations` | `ssr` | yes (`RUSTDOCFLAGS`) |
| `uf-help`, `uf-appearance`, `uf-apps`, `uf-welcome`, `uf-notifications` | (none / `ssr`) | optional, no deny |
| `uf-component-preview` | `ssr` | optional, no deny |

```bash
RUSTDOCFLAGS="-D rustdoc::broken-intra-doc-links" cargo doc \
  -p uf-product-macros \
  -p uf-codegen \
  --no-deps

RUSTDOCFLAGS="-D rustdoc::broken-intra-doc-links" cargo doc \
  -p uf-search-core \
  --features ssr \
  --no-deps

RUSTDOCFLAGS="-D rustdoc::broken-intra-doc-links" cargo doc \
  -p uf-product \
  -p uf-integrations \
  --features ssr \
  --no-deps
```

Optional app / preview crates (no `RUSTDOCFLAGS` deny):

```bash
cargo doc -p uf-apps -p uf-welcome -p uf-help -p uf-appearance -p uf-notifications --no-deps
cargo doc -p uf-component-preview --features ssr --no-deps
```

Enable `ssr` when documenting crates whose docs link SSR-only items. Do not claim
workspace-wide link denial: only the `RUSTDOCFLAGS` invocations above enforce it.
