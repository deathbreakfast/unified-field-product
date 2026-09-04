//! Fixture `uf_app!` for uf-codegen discovery. Parsed as source by the example; not built by the product workspace.

uf_app! {
    name: "Sample Beacon",
    id: "sample-beacon",
    description: "Fixture app for emit_routes_table",
    icon: "Cube",
    version: "0.1.0",
    routes: SampleBeaconRoutes,
    route_path: "/sample-beacon",
}
