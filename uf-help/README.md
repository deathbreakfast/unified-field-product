# uf-help

Optional Help product offering for Unified Field shells: spotlight tours, the
app-bar Help menu, visit progress (Valence + signed-out `localStorage`), and
GitHub filing against each app's `uf_app!` `repository`.

API reference: `cargo doc -p uf-help --features ssr --open`.

## Concern → API

| Concern | API |
|---------|-----|
| Author a step | `help_spotlight_step` + matching DOM `id` for `spotlight` |
| Link inventory | App `ensure_help_steps_linked()`; `uf_help::ensure_linked` |
| Mount player | `HelpTourPlayer` via `uf-integrations` `offering-help` |
| Match route / pending | `route_matches`, `compute_pending`, stable `feature_highlight` |
| Visits | `server` fns (signed-in); `LOCAL_STORAGE_KEY` (signed-out) |
| Replay current route | Help menu, `request_replay_current_route` |
| Errors | `HelpError` → `ServerFnError::ServerError` (message string on wire) |

## Authoring ladder

### 1. Highlight

Register a Leptos body with `#[help_spotlight_step]` and, when you want a cutout,
set `spotlight` to the same string as the target element's HTML `id`.

```rust,ignore
use leptos::prelude::*;
use uf_help_macros::help_spotlight_step;

#[help_spotlight_step(
    route = "/apps",
    feature_highlight = "apps-search",
    title = "Search apps",
    spotlight = "apps-search-input",
    order = 10,
)]
#[component]
pub fn AppsSearchHelp() -> impl IntoView {
    view! {
        <input id="apps-search-input" />
        <p data-testid="help-step-apps-search">"Use search to find installed apps by name."</p>
    }
}
```

Keep `feature_highlight` keys stable. A new key on an existing route shows only
that step to returning users until they finish it.

### 2. Mid — force-link inventory

Call an empty `ensure_help_steps_linked()` from your app crate so `inventory`
submissions are not dropped at link time:

```rust,ignore
uf_apps::ensure_help_linked();
```

See `uf-apps/src/help_steps.rs` for seeded steps on `/apps`.

### 3. Mount

Enable `uf-integrations` feature `offering-help` (or `full`). The stock
`UnifiedFieldShellLayout` mounts `HelpTourPlayer`; call `uf_help::ensure_linked`
for the default app-bar Help utility.

### 4. Runtime

- **Routes** — exact pathname match, or `:param` segments (one non-empty path
  segment each), for example `/apps/:app_name` or `/boson/tasks/:task_name/config`.
- **Signed-out** — `uf.help.tour_steps` in `localStorage`; merged into Valence on first authenticated write.
- **Replay** — Help → Replay spotlight tour affects the current route only.
- **Gates** — auto-play pauses while `RequireAuthenticated` empty states are active.

## Examples

| Example | What it shows |
|---------|---------------|
| [`examples/shell-chrome-host`](../examples/shell-chrome-host/) | Default shell + offerings; `cargo check -p shell-chrome-host --features ssr` |
| [`uf-product-ui-e2e/end2end/tests/help_spotlight.spec.ts`](../uf-product-ui-e2e/end2end/tests/help_spotlight.spec.ts) | Tour once, replay, gate skip, apps/welcome coverage |
| [`uf-apps`](../uf-apps/) `help_steps.rs` | Directory + app-detail steps |
| `uf-notifications` `help_steps.rs` | Bell and inbox steps on `/notifications` |

## Verify

```bash
cargo check -p uf-help --features ssr
cargo test -p uf-help --features ssr
cargo check -p shell-chrome-host --features ssr
```

## Related

- Shell offerings: [`uf-integrations`](../uf-integrations/) (`offering-help` / `full`)
- Macro crate: [`uf-help-macros`](../uf-help-macros/)
- Playwright host: [`uf-product-ui-e2e`](../uf-product-ui-e2e/)
