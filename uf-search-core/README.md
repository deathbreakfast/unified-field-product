# uf-search-core

Leptos-free types for **in-page / picker** search sources (not AppBar content
index). Client-safe source keys and result items, plus (behind `ssr`) the
provider trait and inventory registry. The per-user workspace content index lives
in `uf_product::workspace_search`, not this crate.
`cargo doc -p uf-search-core --features ssr --open`.

## Features

- **Client DTOs** — `SearchSourceKey`, `SearchSourceItem`
- **SSR registry** — `SearchSourceProvider`, `SearchSourceDescriptor`, `SearchSourceRegistry::query_many`

## Usage

```rust,ignore
use uf_search_core::{SearchSourceDescriptor, SearchSourceFuture, SearchSourceItem, SearchSourceProvider};

struct MyProvider;
impl SearchSourceProvider for MyProvider {
    fn query<'a>(
        &'a self,
        valence: &'a valence::Valence,
        query_text: &'a str,
        max_results: u32,
    ) -> SearchSourceFuture<'a> {
        Box::pin(async move { Ok(Vec::<SearchSourceItem>::new()) })
    }
}

static PROVIDER: MyProvider = MyProvider;
quark::inventory::submit! {
    SearchSourceDescriptor {
        id: "my-source",
        label: "My Source",
        description: "Searches my stuff",
        provider: &PROVIDER,
    }
}
```

Prefer `define_search_sources!` from `uf-product-macros` when declaring several
sources. Wire keys into `uf-integrations::SearchSourcePicker`.

Runnable stub: [`examples/query_many_stub.rs`](examples/query_many_stub.rs)
(`cargo run -p uf-search-core --example query_many_stub --features ssr`).

## Verify

```bash
cargo test -p uf-search-core --features ssr
cargo run -p uf-search-core --example query_many_stub --features ssr
cargo doc -p uf-search-core --features ssr --no-deps
```

## Related

- UI picker: [`uf-integrations`](../uf-integrations/)
- Registration macro: [`uf-product-macros`](../uf-product-macros/)
- Product re-export: [`uf-product`](../uf-product/) `search_sources`
