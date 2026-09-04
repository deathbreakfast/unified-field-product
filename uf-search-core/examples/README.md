# uf-search-core examples

| Example | When to use | Command | Success | Look next |
|---------|-------------|---------|---------|-----------|
| [`query_many_stub`](query_many_stub.rs) | Fan-out `SearchSourceRegistry::query_many` | `cargo run -p uf-search-core --example query_many_stub --features ssr` | Stdout: `query_many_stub: OK` with `Beacon Alpha` | `SearchSourcePicker` in `shell-chrome-host` |
