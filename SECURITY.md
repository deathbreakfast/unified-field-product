# Security Policy

## Supported versions

Security fixes are accepted against the latest published `0.3.x` release line of this repository's crates (`uf-product` and related product crates).

## Reporting a vulnerability

Please **do not** open a public GitHub issue for security-sensitive reports.

Prefer one of the following:

1. **GitHub Security Advisories** — use [Report a vulnerability](https://github.com/unified-field-dev/unified-field-product/security/advisories/new) on this repository when available.
2. Contact the maintainers privately via the repository owner listed at https://github.com/unified-field-dev/unified-field-product.

Include:

- a description of the issue and its impact
- steps to reproduce or a proof of concept when possible
- affected crate names and versions

We will acknowledge receipt as soon as practical and coordinate a fix and disclosure timeline with you.

## Scope

In scope: vulnerabilities in this repository's published crates and documentation that could cause unsafe production defaults, plus CI/supply-chain issues in this repository.

Out of scope: vulnerabilities solely in third-party dependencies unless this project mishandles them in a security-relevant way.

## Product authz map (summary)

- **Session / UserAppearance** — owner-scoped via `OWNER_BY_USER_FIELD` on a `user` Record field for read, create, and update; delete is `SYSTEM_ONLY`. Appearance server fns use session `valence()` (aligned with lepton-identity).
- **Notification** — owner read/update; create/delete `SYSTEM_ONLY` (no authenticated create for arbitrary recipients). Action URLs use the same sanitizer as Orbital post-auth referers.
- **Signed-in apps** — Welcome, Apps, and Notifications wrap content in `RequireAuthenticated`.
- **Permission gates** — named permission checks fail closed when Gauge is not wired (deny). UI gates treat only an explicit allow as pass; `None` / error deny.
- **Public by design** — `uf-apps` directory server fns (`get_apps` / `get_apps_page` / `get_app_overview`) return in-memory registry metadata only.
- **Dev-only** — `create_test_notification` requires the `dev-tools` feature and is not enabled in production defaults.
- **Preview search** — `preview_search_principals` requires a signed-in session, uses session Valence (not System), and caps per-source result limits.
- **Telemetry** — `record_page_view` truncates client-supplied field strings before Spectra emit.
- **Welcome featured** — CUD uses session Valence + `WelcomeAdmin` Valence gate (`SYSTEM_ONLY` retained for bootstrap). No mid-request System elevate (`uf-welcome` `no_elevate_path_gate`).
- **Workspace search writer** — index CUD is `OWNER_BY_USER_FIELD` (+ `SYSTEM_ONLY` for job backfill); hard delete asserts ownership before raw backend delete.

## Surviving System exceptions

| Site | Rationale |
|---|---|
| IndexedDemo backfill `should_run` | Prefer job System Valence; rebind only when the iter is not already System |
