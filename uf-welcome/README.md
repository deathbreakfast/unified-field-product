# uf-welcome

Signed-in welcome landing for Unified Field product hosts. Mounted at `/welcome`
for authenticated users: featured apps plus recent / most-used / popular shortcuts
from Spectra page views. Operators with `WelcomeAdmin` manage featured apps at
`/welcome/admin`.

Routes and types: `cargo doc -p uf-welcome --open`.

## Features

- Welcome page cards: Featured, Recent, My most used, Popular
- Help spotlights on those four cards plus Featured "View all apps" (`help_steps`)
- Featured catalog (`WelcomeFeaturedApp`) + `/welcome/admin`
- Permission manifest: `WelcomeAdmin`
- Registers as the `"welcome"` app via `uf_app!` at `/welcome`

## Host wiring

1. Mount [`UfWelcomeRoutes`](src/lib.rs) inside `<Routes>`.
2. Mount `uf_product::PageViewTracker` beside `AppearanceThemeController`.
3. `provide_context(spectra)` so usage server fns can query page views.
4. Force-link `uf_welcome` on the SSR host so Valence schema inventory registers.
5. Enable `uf-welcome/admin-permissions` when Gauge is available for `WelcomeAdmin` checks.

Teaching mount: [`examples/shell-chrome-host`](../examples/shell-chrome-host/).

## Verify

```bash
cargo check -p uf-welcome --features ssr
cargo check -p uf-welcome --features hydrate
cargo test -p uf-welcome --features ssr
cargo test -p uf-product --features ssr usage
cargo doc -p uf-welcome --no-deps
```

## Related

- Page-view emit + usage aggregators: [`uf-product` telemetry](../uf-product/src/telemetry/)
- Apps directory: [`uf-apps`](../uf-apps/)
