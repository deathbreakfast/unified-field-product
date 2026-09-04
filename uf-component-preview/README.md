# Orbital Component Preview (`uf-component-preview`)

Component preview and documentation host for Orbital UI components in Unified Field.

## Purpose

Development catalog at `/orbital` for Orbital components (and a few Unified Field
integrations). Pages use docs and props from `#[component_doc]`, plus registry-driven
slug routes.
Catalog routes and types: `cargo doc -p uf-component-preview --open`.

## Features

- **Component preview pages** — one route per registered slug
- **Component documentation** — docs and props from `#[component_doc]`
- **Registry-driven catalog** — `PreviewCatalog` merges Orbital’s baseline with UF manuals
- **Manual escape hatch** — Search Source Picker and Unified Field icons stay hand-written manuals in this workspace
- **Host-extended zone previews** — History Timeline and Tag Catalog Picker register in their owning crates; hosts call [`extend_registrations`](src/preview/extensions.rs) (wire from the host once those crates are on `unified-field-dev`)
- **Teaching example** — `DemoStatusPill` (`demo-status-pill`) shows the registration path end to end
- **Dev layout** — catalog shell for browsing components

## Host this catalog

Mount [`OrbitalComponentRoutes`](src/routes.rs) in your host (workspace teaching host:
[`examples/component-preview-host`](../examples/component-preview-host/)):

```rust,ignore
use leptos_router::components::Routes;
use uf_component_preview::OrbitalComponentRoutes;

view! {
    <Routes fallback=|| "not found">
        <OrbitalComponentRoutes />
    </Routes>
}
```

The app also registers via `uf_app!`:

```rust
uf_app! {
    name: "Orbital",
    id: "orbital",
    description: "Component library and preview tool for Orbital UI components",
    icon: "🧩",
    version: "0.1.0",
    routes: OrbitalComponentRoutes,
    route_path: "/orbital",
}
```

Routes:

- `/orbital` — catalog index / introduction
- `/orbital/{slug}` — registry preview pages (e.g. `/orbital/demo-status-pill`)

### Expose your own component

1. Annotate with `#[component_doc(..., preview_slug = "...")]` in the owning crate (crate `preview` feature on).
2. Export with `preview_registrations! { &YOUR_PREVIEW_REGISTRATION, … }`.
3. In the host binary, call `uf_component_preview::preview::extend_registrations(your_crate::preview::all())` before serving routes.
4. Rebuild the host — the slug shows under left nav and at `/orbital/{slug}`.

In-repo walkthrough widget: [`src/components/examples/demo_status_pill.rs`](src/components/examples/demo_status_pill.rs).

### Catalog composition

Collection goes through `orbital_primitives::preview::PreviewCatalog` in
`src/preview/registry.rs`: Orbital leaf tables + `uf_product` locals + Tier C manuals +
teaching examples + optional host extensions. New product widgets should use
`#[component_doc]` and `preview_registrations!` in the owning crate, then
`extend_registrations` from the host — not path deps on zone apps.

## Preview surfaces

### Basic

- Text, Scroll Area, Infinite Scroll, Stat Card, Stepper, Paginator, Empty State, Auto Grid
- History Timeline and Tag Catalog Picker (host `extend_registrations` + zone crate `preview` feature)
- Search Source Picker, Unified Field icons
- Demo Status Pill (teaching example)

### Patterns / layout examples

Identity Card, marketing sections, and layout placeholders under `components/layouts`.

Orbital customer recipe for external product catalogs:
[component-testing.md](https://github.com/unified-field-dev/orbital/blob/main/docs/component-testing.md).
