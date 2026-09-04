# uf-codegen examples

| Example | When to use | Command | Success | Look next |
|---------|-------------|---------|---------|-----------|
| [`emit_routes_table`](emit_routes_table.rs) | See `generate_registered_routes` output from a tiny `uf_app!` fixture | `cargo run -p uf-codegen --example emit_routes_table` | Stdout: `emit_routes_table: OK` with `sample-beacon` in generated files | Host `build.rs` in a product app; runtime: `uf_app_registration` |
