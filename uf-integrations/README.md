# uf-integrations

Shell integrations for Unified Field product hosts: app bar, shell layout,
search picker, and standard empty pages.
`cargo doc -p uf-integrations --features ssr --open`.

## Features

- **App bar** — branding, breadcrumbs, search slot, and a generic utilities slot
  (default = Help / Apps / Appearance offerings when `full` is enabled)
- **Offerings** — Cargo features `offering-help`, `offering-apps`,
  `offering-appearance`, `offering-notifications`, and `full` (default).
  `offering-apps` and `offering-notifications` are markers: also depend on
  [`uf-apps`](../uf-apps/) / [`uf-notifications`](../uf-notifications/) so
  inventory registers the Apps button and shell bell.
- **Shell layout** — Orbital `Layout` with app-bar / left-nav slots and permission toast bus. With a left nav, the shell uses `SidebarPresentation::Auto` (drawer below `Md`, inline column on wide viewports) and starts with the sidebar closed.
- **Search** — `SearchSourcePicker` over `uf-search-core` keys and items
- **Pages** — coming-soon and not-found surfaces

## Usage

```rust,ignore
use leptos::prelude::*;
use lepton_shell::AppBarUserMenu;
use uf_integrations::{
    provide_shell_auth_menu, HostAuthMenu, ShellAppBar, ShellAuthMenu,
    UnifiedFieldAppBar, UnifiedFieldShellLayout,
};

provide_shell_auth_menu(|| view! { <AppBarUserMenu /> });

#[component]
fn AppShell(children: Children) -> impl IntoView {
    view! {
        <UnifiedFieldShellLayout>
            <ShellAppBar slot>
                <UnifiedFieldAppBar app_name="My App".to_string()>
                    <ShellAuthMenu slot:auth_menu>
                        <HostAuthMenu />
                    </ShellAuthMenu>
                </UnifiedFieldAppBar>
            </ShellAppBar>
            {children()}
        </UnifiedFieldShellLayout>
    }
}
```

Pass `<AppBarUtilities slot>...</AppBarUtilities>` on `UnifiedFieldAppBar` to replace
the default offering pack with host children.

Runnable host with layout, app bar, search picker, coming-soon, and 404 on one tree:
[`examples/shell-chrome-host`](../examples/shell-chrome-host/).

## Verify

```bash
cargo check -p uf-integrations --features ssr
cargo check -p shell-chrome-host --features ssr
cargo doc -p uf-integrations --features ssr --no-deps
```

## Related

- Session and route guards: [`uf-product`](../uf-product/)
- Help / Appearance offerings: [`uf-help`](../uf-help/), [`uf-appearance`](../uf-appearance/)
- Apps directory + Apps button: [`uf-apps`](../uf-apps/)
- Search DTOs / registry: [`uf-search-core`](../uf-search-core/)
- Search source macros: [`uf-product-macros`](../uf-product-macros/)
