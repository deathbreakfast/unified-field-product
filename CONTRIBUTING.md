# Contributing to Unified Field Product

Thank you for improving this project.

## Development setup

1. Clone [unified-field-dev/unified-field-product](https://github.com/unified-field-dev/unified-field-product)
2. Install Rust stable
3. From the repository root:

```bash
cargo check --workspace
```

## Code of conduct

Participation is governed by [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md). Security reports: [`SECURITY.md`](SECURITY.md).

## Pull requests

- Prefer small, focused PRs.
- Update [`README.md`](README.md) when user-facing flows or host mounting steps change.
- When you change public APIs, update crate rustdoc and run the doc checks in
  [`docs/VERIFICATION.md`](docs/VERIFICATION.md).

## Documentation gates

Workspace `Cargo.toml` sets `missing_docs = "deny"` and
`broken_intra_doc_links = "allow"`. Several UI crates still carry a narrow
`#![allow(missing_docs)]` (Leptos props / design-system re-exports); do not widen those
allows for ordinary new modules.

Broken-link **denial** is applied only when you pass
`RUSTDOCFLAGS="-D rustdoc::broken-intra-doc-links"` as in
[`docs/VERIFICATION.md`](docs/VERIFICATION.md). Packages in that primary gate:

- `uf-product-macros`, `uf-codegen` (no feature extras)
- `uf-search-core`, `uf-product`, `uf-integrations` (with `--features ssr`)

Optional without link-deny: `uf-apps`, `uf-welcome`, `uf-component-preview`
(preview needs `--features ssr`). See VERIFICATION for the exact commands.
