# uf-notifications (product UI)

App-bar notification bell, unread dropdown, and auth-gated `/notifications` inbox
for Unified Field hosts. Persistence and `#[server]` ops live in the domain repo
[uf-notifications](https://github.com/unified-field-dev/uf-notifications)
(`uf-notifications-core`, `uf-notifications-api`).

## Features

- **Notification bell** — Live unread badge and preview dropdown for the app bar.
  Teaching path: crate rustdoc `#mount-shell-bell` (`cargo doc -p uf-notifications --features ssr,lazy-routes --open`).
- **Inbox routes** — Nest `/notifications` under the host router for the full inbox
  (`#mount-inbox-routes` in the same rustdoc).
- **Lazy inbox route** — Optional WASM code-split of the inbox leaf (`lazy-routes`;
  `#lazy-routes` in rustdoc).

## Mount

1. Depend on this crate plus domain core/api with matching `ssr` / `hydrate`.
2. Enable `uf-integrations` `offering-notifications` (or `full`) and call
   `ensure_notification_bell_linked()` once at `App` root.
3. Nest `<NotificationsRoutes />` under your host `<Routes>`.
4. Mount Photon `/ws/notifications` when you want live badge refresh.

Override the inventory bell only with
`uf_integrations::provide_shell_notification_bell`.

Crate-root rustdoc owns Features vs Feature flags and the full get-started guides.

## Verify

See [docs/VERIFICATION.md](../docs/VERIFICATION.md) (`cargo check -p uf-notifications --features ssr`
and Playwright `notifications.spec.ts` under `uf-product-ui-e2e`).
