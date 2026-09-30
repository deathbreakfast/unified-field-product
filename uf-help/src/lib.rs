//! Optional Help product offering for Unified Field shells.
//!
//! Ships the app-bar Help menu (bug / feature / security reports and replay),
//! a spotlight tour player backed by compile-time step inventory, visit progress
//! in Valence plus a signed-out `localStorage` mirror, and a GitHub feedback bot.
//! Help center CMS routes, Orbital spotlight primitives, platform anon identity
//! claims, and Neutrino secret wiring live in other crates.
//!
//! ## Features
//!
//! - **Spotlight step authoring** — Register Leptos help bodies as inventory steps with
//!   stable `feature_highlight` keys and optional DOM cutouts. [Get started](#author-a-spotlight-step)
//! - **Help tour player** — Stock [`HelpTourPlayer`] auto-plays pending spotlight steps
//!   beside shell chrome when `uf-integrations` feature `offering-help` is enabled.
//!   Anchored steps [wait until their element is on screen](#steps-for-ui-that-appears-later).
//!   [Get started](#mount-tour-player)
//! - **App-bar Help control** — [`AppBarHelpButton`] opens report dialogs and replay for the current route.
//! - **Visit progress** — Valence rows when signed in; [`LOCAL_STORAGE_KEY`] mirror when signed out.
//! - **GitHub feedback bot** — Bug, feature, and security report dialogs submit via [`GitHubFeedbackClient`].
//!
//! ## Getting started
//!
//! ```bash
//! cargo doc -p uf-help --features ssr --open
//! cargo check -p uf-help --features ssr
//! ```
//!
//! Typical integration order: author steps with [`help_spotlight_step`], force-link inventory
//! in the host binary, then enable `offering-help` on `uf-integrations` so [`HelpTourPlayer`]
//! mounts automatically.
//!
//! ## Author a spotlight step
//!
//! [`help_spotlight_step`] registers a Leptos component as a Help tour step via `inventory`.
//! Each step binds a route pathname, stable [`HelpStepDescriptor::feature_highlight`] key,
//! optional spotlight DOM `id`, and panel copy. App crates define steps in `help_steps.rs`
//! modules and call `ensure_help_steps_linked()` so submissions survive linking.
//!
//! **Prerequisites:** `uf-help` and `uf-help-macros` in the app crate; target UI element with
//! matching HTML `id` when `spotlight` is set.
//!
//! ```rust,ignore
//! use leptos::prelude::*;
//! use uf_help::help_spotlight_step;
//!
//! #[help_spotlight_step(
//!     route = "/apps",
//!     feature_highlight = "apps-search",
//!     title = "Search apps",
//!     spotlight = "apps-search-input",
//!     order = 10,
//! )]
//! #[component]
//! pub fn AppsSearchHelp() -> impl IntoView {
//!     view! {
//!         <input id="apps-search-input" />
//!         <p data-testid="help-step-apps-search">"Use search to find apps."</p>
//!     }
//! }
//!
//! /// Call from the app crate so inventory is linked into the host binary.
//! pub fn ensure_help_steps_linked() {}
//! ```
//!
//! On success the step appears in [`collect_help_steps_for_route`] for `/apps` and Playwright
//! can assert `data-testid="help-step-apps-search"`. Keep `feature_highlight` stable across
//! releases; renaming creates a new highlight key for returning users.
//!
//! ## Mount tour player
//!
//! [`HelpTourPlayer`] reads inventory for the current pathname, resolves visit rows (Valence
//! when signed in, local storage when signed out), and opens Orbital spotlight panels for
//! pending steps. Enable once at host boot by depending on `uf-integrations` with feature
//! `offering-help` (or `full`); the stock `uf_integrations::UnifiedFieldShellLayout` mounts
//! [`HelpTourPlayer`] beside page chrome. Call [`ensure_linked`] if you rely on uf-help's
//! app-bar utility registration.
//!
//! **Prerequisites:** `ssr` and/or `hydrate` on `uf-help` and `uf-integrations`; step inventory
//! force-linked in the host binary; session context from `uf-product` for signed-in visits.
//!
//! ```rust,ignore
//! // Cargo.toml: uf-integrations = { workspace = true, features = ["offering-help"] }
//! use leptos::prelude::*;
//! use uf_help::{ensure_linked, HelpTourPlayer};
//! use uf_integrations::UnifiedFieldShellLayout;
//!
//! // Once at host boot, before routed pages mount:
//! ensure_linked();
//!
//! #[component]
//! fn AppShell(children: Children) -> impl IntoView {
//!     view! {
//!         <UnifiedFieldShellLayout>
//!             <HelpTourPlayer />
//!             {children()}
//!         </UnifiedFieldShellLayout>
//!     }
//! }
//! ```
//!
//! On success the shell renders spotlight chrome and pending steps auto-play unless
//! [`uf_product::AccessGateActive`] suppresses tours during auth gates. Replay from
//! Help → Replay spotlight tour scopes to the current route only ([`help_request_replay_for_route`]).
//!
//! ## Steps for UI that appears later
//!
//! Give a step a `spotlight` id when it explains UI that isn't on screen at first: a panel
//! behind a button, a wizard stage, or a section that only renders for some records. The
//! player shows the step once the element is visible and leaves it pending until then, so
//! one route can carry steps for every state of the page.
//!
//! **Prerequisites:** the same setup as [Author a spotlight step](#author-a-spotlight-step),
//! with the `spotlight` id on the element that appears later.
//!
//! ```rust,ignore
//! use leptos::prelude::*;
//! use uf_help::help_spotlight_step;
//!
//! #[help_spotlight_step(
//!     route = "/reports",
//!     feature_highlight = "reports-panel",
//!     title = "Report panel",
//!     spotlight = "report-panel",
//!     order = 20,
//! )]
//! #[component]
//! pub fn ReportPanelHelp() -> impl IntoView {
//!     view! { <p data-testid="help-step-reports-panel">"Filters and exports live here."</p> }
//! }
//!
//! #[component]
//! pub fn ReportsPage() -> impl IntoView {
//!     let open = RwSignal::new(false);
//!     view! {
//!         <button on:click=move |_| open.set(true)>"Show report"</button>
//!         <Show when=move || open.get()>
//!             <section id="report-panel">"…"</section>
//!         </Show>
//!     }
//! }
//! ```
//!
//! On first visit the tour skips `reports-panel` and records only the steps it showed. After
//! **Show report**, the player opens a tour with just that step. Until the panel mounts, the
//! key has no visit row, so it stays pending across reloads. Replay follows the same rule and
//! replays only the steps whose elements are on screen.
//!
//! ## Runtime semantics
//!
//! - **Route matching** — step `route` is exact pathname equality, or a
//!   `:param` segment pattern that matches one non-empty path segment per
//!   parameter (for example `"/apps/:app_name"`, `"/boson/tasks/:task_name"`).
//!   Inventory `/permission/permissions` also matches bare `/permission` (same
//!   Permission index page). Valence rows store the inventory pattern, not the
//!   live slug ([`inventory_route_keys_for_pathname`]).
//! - **Pending** — no visit row, or visit with `replay == true` ([`compute_pending`]).
//! - **Visible anchors** — a pending step with a `spotlight` id is shown only when that
//!   element exists, has a non-zero box, and isn't `visibility: hidden`. Steps without an id
//!   always show. While anchored steps wait, the player re-checks every 400 ms and opens once
//!   the visible set holds steady across two checks. Only shown steps are marked seen; a
//!   route change closes an open tour without marking it.
//! - **Signed-out** — progress lives under [`LOCAL_STORAGE_KEY`]; on first signed-in
//!   write, [`help_mark_steps_seen`] merges missing local rows into Valence
//!   ([`merge_local_into_server`] on read keeps local-only rows visible until then).
//! - **Replay** — Help → Replay spotlight tour sets replay for the **current route
//!   only** ([`help_request_replay_for_route`] / [`local_request_replay_for_route`]).
//!
//! Server failures map through [`HelpError::into_server_fn_error`]: callers receive
//! [`ServerFnError::ServerError`](leptos::prelude::ServerFnError) with the
//! [`Display`](std::fmt::Display) message only (no structured variant on the wire).
//! See [`HelpError`] and server function `# Errors` sections on [`help_list_visits_for_route`].
//!
//! ## Examples
//!
//! | Example | What it shows |
//! |---------|---------------|
//! | `examples/shell-chrome-host/` | Shell layout with default offerings; `cargo check -p shell-chrome-host --features ssr` |
//! | `uf-product-ui-e2e/end2end/tests/help_spotlight.spec.ts` | Once (anon/authed), replay current route, access-gate skip, apps/welcome steps, steps that wait for their anchor on `/help-fixture/deferred` |
//! | `uf-apps` `help_steps.rs` | Seeded `/apps` steps + `uf_apps::ensure_help_linked` |
//! | `uf-notifications` `help_steps.rs` | Bell / inbox steps on `/notifications` (link inventory the same way) |
//!
//! ## Where to look next
//!
//! - [`HelpStepDescriptor`] / [`collect_help_steps_for_route`] — inventory collection and route matching.
//! - [`HelpTourPlayer`] — spotlight player component.
//! - [`AppBarHelpButton`] — app-bar Help menu and replay trigger.
//! - [`HelpReportDialog`] — bug / feature / security report UI.
//! - [`help_list_visits_for_route`], [`help_mark_steps_seen`], [`help_request_replay_for_route`] — server fns.
//! - `uf-help-macros` — [`help_spotlight_step`] attribute reference.
//! - `uf-integrations` — `UnifiedFieldShellLayout` and `offering-help` feature flag.

#![allow(missing_docs)]
#![deny(clippy::missing_errors_doc)]

pub use inventory;
pub use uf_help_macros::help_spotlight_step;

mod app_bar_help;
mod error;
mod github;
mod report;
mod repository;
mod server;
mod service;
#[cfg(feature = "ssr")]
mod service_reports;
mod step_inventory;
mod tour;

pub use app_bar_help::{AppBarHelpButton, APP_BAR_UTILITY_ORDER};
pub use error::HelpError;
pub use github::{
    clear_github_token_resolver, set_github_token_resolver, BugReportPayload, CreateIssue,
    FeatureRequestPayload, GitHubFeedbackClient, HttpGitHubClient, MockGitHubClient,
    PrivateVulnReport, SecurityReportPayload, HELP_GITHUB_FEEDBACK_SECRET,
    UF_HELP_GITHUB_TOKEN_ENV,
};
pub use report::{HelpReportDialog, HelpReportKind};
pub use repository::{parse_github_owner_repo, resolve_help_repository};
pub use server::{
    help_list_visits_for_route, help_mark_steps_seen, help_pending_steps_for_route,
    help_repository_for_route, help_request_replay_for_route, submit_help_bug_report,
    submit_help_feature_request, submit_help_security_report, HelpPendingStepDto,
};
pub use service::{
    apply_replay_for_route, compute_pending, local_mark_steps_seen, local_request_replay_for_route,
    merge_local_into_server, read_local_visits, read_local_visits_for_route, replay_from_stored,
    replay_to_stored, write_local_visits, HelpStepKey, HelpVisitRecord, LOCAL_STORAGE_KEY,
};
pub use step_inventory::{
    collect_help_steps, collect_help_steps_for_route, inventory_route_keys_for_pathname,
    route_matches, HelpStepDescriptor,
};
pub use tour::{notify_help_replay, request_replay_current_route, HelpTourPlayer};

use uf_product::register_app_bar_utility;

/// Ensure this crate's inventory submissions are linked (call from hosts if needed).
pub fn ensure_linked() {
    register_app_bar_utility();
}
