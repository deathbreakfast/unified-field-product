//! New-build detection for tabs that stay open across a deploy.
//!
//! A release serves its WASM, JS glue and CSS from a stamped directory such as
//! `pkg-20261125093000-0123456789ab` (`LeptosOptions::site_pkg_dir`). That name is
//! the build id. When a host mounts [`build_version_routes`], the shell
//! compares the id the open page loaded with the server's current one. When
//! they differ, it downloads the new build in the background, then shows a
//! "New version available" bar. When a server function has already failed with
//! a deserialisation error, it shows a reload dialog instead. After the reload,
//! a toast confirms the switch.
//!
//! A fresh page load always runs the build its HTML names. The service worker
//! only caches the stamped `/pkg-…/` files, so repeat visits and the reload
//! after a deploy skip the download. It never caches HTML, server functions or
//! anything else.
//!
//! Hosts that don't mount the routes, or serve an unstamped `pkg` dir (local
//! dev), get none of this: no `/_uf/build.json` requests and no service worker.
//!
//! # Concern → API
//!
//! | Concern | API |
//! |---------|-----|
//! | Serve `/_uf/build.json` and `/uf-sw.js` (server) | [`build_version_routes`] |
//! | Shell state, watcher and bar (mounted by `UnifiedFieldShellLayout`) | [`provide_build_version_state`], [`BuildVersionWatcher`], [`BuildVersionBar`] |
//! | Feed a failed server function into the check | [`report_build_version_error`] |
//! | Pretend the page runs an older build (e2e only) | `BuildOverride` (`e2e-build-override` feature) |
//!
//! # Example
//!
//! ```rust,ignore
//! let app = Router::new()
//!     .merge(uf_product::build_version::build_version_routes(&leptos_options))
//!     .leptos_routes(&leptos_options, routes, App);
//! ```

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::components::{Body1, Caption1};
use crate::primitives::{
    use_breakpoint_down, Breakpoint, Button, ButtonAppearance, ButtonSize, Dialog, DialogActions,
    DialogBody, DialogContent, DialogSurface, DialogTitle, Flex, FlexGap, MessageBar,
    MessageBarActions, MessageBarBody, MessageBarIntent, MessageBarLayout, MessageBarTitle,
    Spinner, SpinnerSize,
};

#[cfg(feature = "ssr")]
mod routes;
#[cfg(all(feature = "hydrate", not(feature = "ssr")))]
mod watcher;

#[cfg(all(feature = "ssr", feature = "e2e-build-override"))]
pub use routes::BuildOverride;
#[cfg(feature = "ssr")]
pub use routes::{build_version_routes, read_build_manifest};

/// Path of the build manifest served by [`build_version_routes`].
pub const BUILD_JSON_PATH: &str = "/_uf/build.json";
/// Path of the pkg-cache service worker served by [`build_version_routes`].
pub const SERVICE_WORKER_PATH: &str = "/uf-sw.js";
/// `sessionStorage` key holding the build a tab reloaded away from.
pub const UPDATED_FROM_KEY: &str = "uf:updated-from";
/// Minimum gap between route-change (and interval) checks.
pub const ROUTE_CHECK_THROTTLE_MS: f64 = 60_000.0;
/// How often a visible page re-checks the build.
pub const VISIBLE_CHECK_INTERVAL_MS: f64 = 600_000.0;
/// How long "Not now" hides the bar when the route doesn't change.
pub const SNOOZE_MS: f64 = 1_800_000.0;

/// Attribute on the hidden marker the shell renders when the routes are mounted.
#[cfg_attr(not(all(feature = "hydrate", not(feature = "ssr"))), allow(dead_code))]
const WATCH_MARKER_ATTR: &str = "data-uf-build-watch";
/// Marker attribute carrying an e2e override for the running build id.
#[cfg_attr(not(all(feature = "hydrate", not(feature = "ssr"))), allow(dead_code))]
const RUNNING_OVERRIDE_ATTR: &str = "data-uf-build-running";

/// Body of `GET /_uf/build.json`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildManifest {
    /// Current build id: the server's `site_pkg_dir`.
    pub build: String,
    /// Absolute paths of every file in the current pkg dir.
    pub assets: Vec<String>,
}

/// The build id the open page loaded, read from its module script URL.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RunningBuild {
    /// A release build served from `pkg-<stamp>`.
    Stamped(String),
    /// An unstamped `pkg` dir (local dev, e2e): not versioned.
    Unstamped,
    /// No pkg path found in the URL.
    Missing,
}

/// True for `pkg` and `pkg-<letters, digits and dashes>`.
#[must_use]
pub fn is_valid_build_id(id: &str) -> bool {
    match id.strip_prefix("pkg") {
        Some("") => true,
        Some(rest) => {
            rest.len() > 1
                && rest.starts_with('-')
                && rest[1..]
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        }
        None => false,
    }
}

/// True for a stamped release id (`pkg-…`), false for `pkg` or anything invalid.
#[must_use]
pub fn is_stamped_build_id(id: &str) -> bool {
    id != "pkg" && is_valid_build_id(id)
}

/// Read the build id from a module script or modulepreload URL (absolute or
/// root-relative): its first path segment.
#[must_use]
pub fn parse_build_id(url: &str) -> RunningBuild {
    let path = match url.find("://") {
        Some(idx) => {
            let after_scheme = &url[idx + 3..];
            after_scheme
                .find('/')
                .map_or("", |slash| &after_scheme[slash..])
        }
        None => url,
    };
    let segment = path
        .split(['?', '#'])
        .next()
        .unwrap_or_default()
        .split('/')
        .find(|s| !s.is_empty())
        .unwrap_or_default();
    if segment == "pkg" {
        RunningBuild::Unstamped
    } else if is_stamped_build_id(segment) {
        RunningBuild::Stamped(segment.to_string())
    } else {
        RunningBuild::Missing
    }
}

/// What asked for a build check.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckTrigger {
    /// First check after hydrate.
    Hydrate,
    /// The route changed.
    Route,
    /// The periodic timer fired.
    Interval,
    /// A server function failed with a version or deserialisation error.
    ServerFnError,
}

/// Decide whether a check should run now.
///
/// Hydrate and server-function checks always run. Route and interval checks
/// run at most once per [`ROUTE_CHECK_THROTTLE_MS`], and interval checks only
/// while the page is visible.
#[must_use]
pub fn should_check(
    trigger: CheckTrigger,
    last_check_ms: Option<f64>,
    now_ms: f64,
    visible: bool,
) -> bool {
    let throttled = last_check_ms.is_some_and(|last| now_ms - last < ROUTE_CHECK_THROTTLE_MS);
    match trigger {
        CheckTrigger::Hydrate | CheckTrigger::ServerFnError => true,
        CheckTrigger::Route => !throttled,
        CheckTrigger::Interval => visible && !throttled,
    }
}

/// True while a "Not now" snooze set to end at `until_ms` still applies.
#[must_use]
pub fn snooze_active(until_ms: Option<f64>, now_ms: f64) -> bool {
    until_ms.is_some_and(|until| now_ms < until)
}

/// True when a server-function error looks like the client and server builds
/// disagree.
#[must_use]
pub fn is_version_error(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("deserializ") || lower.contains("version mismatch")
}

/// Which new-version prompt is showing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BuildPrompt {
    /// Nothing to show.
    #[default]
    Hidden,
    /// A new build is live: the bar offers Reload now.
    Available,
    /// Reload now was pressed and the switch is under way.
    Switching,
    /// The new build couldn't be downloaded.
    Failed,
    /// A server function already failed: the reload dialog is open.
    ReloadDialog,
}

/// Shell-wide new-version state. [`provide_build_version_state`] creates it.
#[derive(Clone, Copy)]
pub struct BuildVersionState {
    prompt: RwSignal<BuildPrompt>,
    snoozed_until: RwSignal<Option<f64>>,
    dialog_open: RwSignal<bool>,
    server_fn_failures: RwSignal<u64>,
    // Read only by the browser-side watcher.
    #[cfg_attr(not(all(feature = "hydrate", not(feature = "ssr"))), allow(dead_code))]
    running: StoredValue<Option<String>>,
    #[cfg_attr(not(all(feature = "hydrate", not(feature = "ssr"))), allow(dead_code))]
    latest: StoredValue<Option<BuildManifest>>,
    #[cfg_attr(not(all(feature = "hydrate", not(feature = "ssr"))), allow(dead_code))]
    prefetched: StoredValue<bool>,
}

impl BuildVersionState {
    fn bar_prompt(self) -> Option<BuildPrompt> {
        let prompt = self.prompt.get();
        let snoozed = self.snoozed_until.get().is_some();
        match prompt {
            BuildPrompt::Available | BuildPrompt::Failed if snoozed => None,
            BuildPrompt::Available | BuildPrompt::Failed | BuildPrompt::Switching => Some(prompt),
            BuildPrompt::Hidden | BuildPrompt::ReloadDialog => None,
        }
    }

    /// "Not now" on the bar or the dialog.
    fn snooze(self) {
        self.dialog_open.set(false);
        if self.prompt.get_untracked() == BuildPrompt::ReloadDialog {
            self.prompt.set(BuildPrompt::Available);
        }
        let now = now_ms();
        let until = now + SNOOZE_MS;
        self.snoozed_until.set(Some(until));
        let snoozed_until = self.snoozed_until;
        set_timeout(
            move || {
                if snoozed_until.get_untracked() == Some(until) {
                    snoozed_until.set(None);
                }
            },
            std::time::Duration::from_millis(SNOOZE_MS as u64),
        );
    }

    #[cfg_attr(
        not(all(feature = "hydrate", not(feature = "ssr"))),
        allow(clippy::unused_self)
    )]
    fn reload_now(self) {
        #[cfg(all(feature = "hydrate", not(feature = "ssr")))]
        watcher::reload_now(self);
    }
}

#[cfg(all(feature = "hydrate", not(feature = "ssr")))]
fn now_ms() -> f64 {
    js_sys::Date::now()
}

#[cfg(not(all(feature = "hydrate", not(feature = "ssr"))))]
fn now_ms() -> f64 {
    0.0
}

/// Create the new-version state and put it in context. `UnifiedFieldShellLayout`
/// calls this once, above [`BuildVersionWatcher`] and [`BuildVersionBar`].
pub fn provide_build_version_state() -> BuildVersionState {
    let state = BuildVersionState {
        prompt: RwSignal::new(BuildPrompt::Hidden),
        snoozed_until: RwSignal::new(None),
        dialog_open: RwSignal::new(false),
        server_fn_failures: RwSignal::new(0),
        running: StoredValue::new(None),
        latest: StoredValue::new(None),
        prefetched: StoredValue::new(false),
    };
    provide_context(state);
    state
}

/// Read the new-version state from context, if a shell provided it.
#[must_use]
pub fn use_build_version_state() -> Option<BuildVersionState> {
    use_context::<BuildVersionState>()
}

/// Ask for a build check when `error` looks like a client/server build
/// mismatch. Returns `true` when it did. Does nothing outside a shell, or when
/// the host hasn't mounted [`build_version_routes`].
///
/// `services::permission_server_errors::report_server_fn_error` already calls
/// this, so callers that report server-fn errors there get it for free.
pub fn report_build_version_error(error: &ServerFnError) -> bool {
    report_build_version_message(&error.to_string())
}

pub(crate) fn report_build_version_message(message: &str) -> bool {
    if !is_version_error(message) {
        return false;
    }
    let Some(state) = use_build_version_state() else {
        return false;
    };
    state.server_fn_failures.update(|n| *n += 1);
    true
}

/// What the server-rendered marker says: `None` when the shell should stay
/// quiet, `Some(running_override)` when it should watch.
#[cfg(feature = "ssr")]
fn watch_marker() -> Option<Option<String>> {
    routes::watch_marker()
}

#[cfg(all(feature = "hydrate", not(feature = "ssr")))]
fn watch_marker() -> Option<Option<String>> {
    let document = web_sys::window()?.document()?;
    let marker = document
        .query_selector(&format!("[{WATCH_MARKER_ATTR}]"))
        .ok()??;
    Some(marker.get_attribute(RUNNING_OVERRIDE_ATTR))
}

#[cfg(not(any(feature = "ssr", feature = "hydrate")))]
fn watch_marker() -> Option<Option<String>> {
    None
}

/// Starts the build check, registers the service worker and renders the
/// reload dialog. Renders nothing visible otherwise.
///
/// Mount once inside `ToasterProvider`, after [`provide_build_version_state`].
#[component]
pub fn BuildVersionWatcher() -> impl IntoView {
    let Some(state) = use_build_version_state() else {
        return ().into_any();
    };
    let marker = watch_marker();

    #[cfg(all(feature = "hydrate", not(feature = "ssr")))]
    {
        watcher::announce_update(marker.clone().flatten());
        if let Some(running_override) = marker.clone() {
            watcher::start(state, running_override);
        }
    }

    let dialog_open = state.dialog_open;
    // Escape or a backdrop click closes the dialog: treat it as Not now.
    Effect::new(move |was_open: Option<bool>| {
        let open = dialog_open.get();
        if was_open == Some(true)
            && !open
            && state.prompt.get_untracked() == BuildPrompt::ReloadDialog
        {
            state.snooze();
        }
        open
    });

    view! {
        {marker.map(|running| view! {
            <span hidden=true data-uf-build-watch="" data-uf-build-running=running></span>
        })}
        <Dialog open=dialog_open>
            <DialogSurface attr:data-testid="uf-build-dialog">
                <DialogBody>
                    <DialogTitle>"Reload to keep going"</DialogTitle>
                    <DialogContent>
                        <Flex vertical=true gap=FlexGap::Small>
                            <Body1>
                                "The site was updated while you had this page open, and this version can't load your data any more. Reload to switch to the new version."
                            </Body1>
                            <Caption1>
                                "Anything you haven't saved on this page will be lost. You'll come back to this same page."
                            </Caption1>
                        </Flex>
                    </DialogContent>
                    <DialogActions>
                        <span data-testid="uf-build-dialog-dismiss">
                            <Button
                                appearance=ButtonAppearance::Secondary
                                on_click=Callback::new(move |_| state.snooze())
                            >
                                "Not now"
                            </Button>
                        </span>
                        <span data-testid="uf-build-dialog-reload">
                            <Button
                                appearance=ButtonAppearance::Primary
                                on_click=Callback::new(move |_| state.reload_now())
                            >
                                "Reload now"
                            </Button>
                        </span>
                    </DialogActions>
                </DialogBody>
            </DialogSurface>
        </Dialog>
    }
    .into_any()
}

/// The "New version available" bar. `UnifiedFieldShellLayout` renders it at
/// the top of the main content; it's empty until a new build is live.
#[component]
pub fn BuildVersionBar() -> impl IntoView {
    let Some(state) = use_build_version_state() else {
        return ().into_any();
    };
    let narrow = use_breakpoint_down(Breakpoint::Md);

    (move || {
        let prompt = state.bar_prompt()?;
        // Below Md the buttons sit under the text.
        let layout = if narrow.get() {
            MessageBarLayout::Multiline
        } else {
            MessageBarLayout::Singleline
        };
        let (intent, title, body) = match prompt {
            BuildPrompt::Failed => (
                MessageBarIntent::Error,
                "Couldn't switch to the new version",
                "Check your connection and try again. You're still on the current version, and it keeps working.",
            ),
            BuildPrompt::Switching => (
                MessageBarIntent::Warning,
                "New version available",
                "Loading the new version. This takes a second or two.",
            ),
            _ => (
                MessageBarIntent::Warning,
                "New version available",
                "The site was updated while you had it open. You can keep working, but some pages may not load or save correctly until you reload.",
            ),
        };
        let actions = if prompt == BuildPrompt::Switching {
            view! {
                <Flex gap=FlexGap::Small attr:data-testid="uf-build-bar-switching">
                    <Spinner size=SpinnerSize::Tiny />
                    <Caption1>"Switching…"</Caption1>
                </Flex>
            }
            .into_any()
        } else {
            let reload_label = if prompt == BuildPrompt::Failed {
                "Try again"
            } else {
                "Reload now"
            };
            view! {
                <span data-testid="uf-build-bar-reload">
                    <Button
                        appearance=ButtonAppearance::Primary
                        size=ButtonSize::Small
                        on_click=Callback::new(move |_| state.reload_now())
                    >
                        {reload_label}
                    </Button>
                </span>
                <span data-testid="uf-build-bar-dismiss">
                    <Button
                        appearance=ButtonAppearance::Secondary
                        size=ButtonSize::Small
                        on_click=Callback::new(move |_| state.snooze())
                    >
                        "Not now"
                    </Button>
                </span>
            }
            .into_any()
        };
        Some(view! {
            <div data-testid="uf-build-bar" data-uf-build-prompt=format!("{prompt:?}")>
                <MessageBar intent=intent layout=layout>
                    <MessageBarTitle>{title}</MessageBarTitle>
                    <MessageBarBody>{body}</MessageBarBody>
                    <MessageBarActions>{actions}</MessageBarActions>
                </MessageBar>
            </div>
        })
    })
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_build_id_reads_stamped_dir() {
        assert_eq!(
            parse_build_id("/pkg-20261125093000-0123456789ab/app.js"),
            RunningBuild::Stamped("pkg-20261125093000-0123456789ab".into())
        );
        assert_eq!(
            parse_build_id("https://example.com/pkg-20261125093000-abc/app_bg.wasm?v=1"),
            RunningBuild::Stamped("pkg-20261125093000-abc".into())
        );
    }

    #[test]
    fn parse_build_id_unstamped_and_missing() {
        assert_eq!(parse_build_id("/pkg/app.js"), RunningBuild::Unstamped);
        assert_eq!(
            parse_build_id("http://localhost:3000/pkg/app.js"),
            RunningBuild::Unstamped
        );
        assert_eq!(parse_build_id(""), RunningBuild::Missing);
        assert_eq!(parse_build_id("/assets/app.js"), RunningBuild::Missing);
        assert_eq!(parse_build_id("https://example.com"), RunningBuild::Missing);
        assert_eq!(parse_build_id("/pkg-/app.js"), RunningBuild::Missing);
        assert_eq!(parse_build_id("/pkg-a_b/app.js"), RunningBuild::Missing);
    }

    #[test]
    fn build_id_validation() {
        assert!(is_valid_build_id("pkg"));
        assert!(is_valid_build_id("pkg-20261125093000-0123456789ab"));
        assert!(!is_valid_build_id("pkg-"));
        assert!(!is_valid_build_id("pkg-../x"));
        assert!(!is_valid_build_id("pkgx"));
        assert!(!is_valid_build_id("assets"));
        assert!(!is_stamped_build_id("pkg"));
        assert!(is_stamped_build_id("pkg-1"));
    }

    #[test]
    fn hydrate_and_server_fn_checks_always_run() {
        for trigger in [CheckTrigger::Hydrate, CheckTrigger::ServerFnError] {
            assert!(should_check(trigger, Some(1_000.0), 1_001.0, false));
            assert!(should_check(trigger, None, 0.0, false));
        }
    }

    #[test]
    fn route_checks_throttle_to_once_a_minute() {
        assert!(should_check(CheckTrigger::Route, None, 0.0, true));
        assert!(!should_check(
            CheckTrigger::Route,
            Some(0.0),
            59_999.0,
            true
        ));
        assert!(should_check(CheckTrigger::Route, Some(0.0), 60_000.0, true));
        // Route checks don't care about visibility.
        assert!(should_check(
            CheckTrigger::Route,
            Some(0.0),
            60_000.0,
            false
        ));
    }

    #[test]
    fn interval_checks_need_a_visible_page() {
        assert!(should_check(
            CheckTrigger::Interval,
            Some(0.0),
            600_000.0,
            true
        ));
        assert!(!should_check(
            CheckTrigger::Interval,
            Some(0.0),
            600_000.0,
            false
        ));
        assert!(!should_check(
            CheckTrigger::Interval,
            Some(0.0),
            30_000.0,
            true
        ));
    }

    #[test]
    fn snooze_window() {
        assert!(!snooze_active(None, 0.0));
        assert!(snooze_active(Some(SNOOZE_MS), SNOOZE_MS - 1.0));
        assert!(!snooze_active(Some(SNOOZE_MS), SNOOZE_MS));
    }

    #[test]
    fn classifies_server_fn_errors() {
        assert!(is_version_error(
            "error deserializing server function results: missing field `id`"
        ));
        assert!(is_version_error(
            "Deserialization error: unexpected end of input"
        ));
        assert!(is_version_error("client version mismatch"));
        assert!(!is_version_error("permission_denied::counter.admin.set"));
        assert!(!is_version_error("error running server function: timeout"));
    }

    #[test]
    fn report_needs_a_shell() {
        let owner = Owner::new();
        owner.with(|| {
            assert!(!report_build_version_message("error deserializing"));
            let state = provide_build_version_state();
            assert!(!report_build_version_message("permission_denied::x"));
            assert!(report_build_version_message("error deserializing"));
            assert_eq!(state.server_fn_failures.get_untracked(), 1);
        });
    }
}
