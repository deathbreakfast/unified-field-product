//! Browser side of the build check: reads the running build, registers the
//! service worker, polls `/_uf/build.json`, prefetches a new build and
//! reloads into it.

use std::time::Duration;

use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    MessageChannel, MessageEvent, RegistrationOptions, RequestCache, RequestInit, Response,
    ServiceWorkerRegistration, VisibilityState,
};

use crate::components::{ToastIntent, ToastOptions, ToasterInjection};

use super::{
    parse_build_id, should_check, snooze_active, BuildManifest, BuildPrompt, BuildVersionState,
    CheckTrigger, RunningBuild, BUILD_JSON_PATH, SERVICE_WORKER_PATH, UPDATED_FROM_KEY,
    VISIBLE_CHECK_INTERVAL_MS,
};

/// How long to wait for the service worker to become ready.
const READY_TIMEOUT_MS: i32 = 15_000;
/// How long a prefetch may take before it counts as failed.
const PREFETCH_TIMEOUT_MS: i32 = 120_000;
/// How long the Updated toast stays up.
const UPDATED_TOAST_MS: i32 = 5_000;

/// Start watching. `running_override` comes from the server-rendered marker
/// (e2e only); otherwise the running build is read from the document head.
pub(super) fn start(state: BuildVersionState, running_override: Option<String>) {
    let Some(running) = running_override.or_else(running_build_from_document) else {
        return;
    };
    state.running.set_value(Some(running.clone()));
    let toaster = use_context::<ToasterInjection>();
    let pathname = leptos_router::hooks::use_location().pathname;
    let last_check = StoredValue::new(None::<f64>);

    // Everything below touches the browser, so it runs once hydration is done.
    Effect::new(move |started: Option<()>| {
        if started.is_some() {
            return;
        }
        show_updated_toast(toaster, &running);
        register_service_worker();
        check(state, last_check, CheckTrigger::Hydrate);

        let interval = set_interval_with_handle(
            move || {
                if !snooze_active(state.snoozed_until.get_untracked(), super::now_ms()) {
                    state.snoozed_until.set(None);
                }
                check(state, last_check, CheckTrigger::Interval);
            },
            Duration::from_millis(VISIBLE_CHECK_INTERVAL_MS as u64),
        );
        if let Ok(handle) = interval {
            on_cleanup(move || handle.clear());
        }
    });

    // A route change ends a "Not now" snooze and asks for a (throttled) check.
    Effect::new(move |prev: Option<String>| {
        let path = pathname.get();
        if prev.as_ref().is_some_and(|prev| *prev != path) {
            state.snoozed_until.set(None);
            check(state, last_check, CheckTrigger::Route);
        }
        path
    });

    Effect::new(move |prev: Option<u64>| {
        let failures = state.server_fn_failures.get();
        if prev.is_some_and(|prev| prev != failures) {
            check(state, last_check, CheckTrigger::ServerFnError);
        }
        failures
    });
}

/// Reload now / Try again: make sure the new build is cached, then reload.
pub(super) fn reload_now(state: BuildVersionState) {
    if state.prompt.get_untracked() == BuildPrompt::Switching {
        return;
    }
    let from_dialog = state.dialog_open.get_untracked();
    state.dialog_open.set(false);
    state.prompt.set(BuildPrompt::Switching);
    state.snoozed_until.set(None);
    spawn_local(async move {
        if !state.prefetched.get_value() {
            let ok = prefetch_latest(state).await;
            state.prefetched.set_value(ok);
            if !ok && !from_dialog {
                state.prompt.set(BuildPrompt::Failed);
                return;
            }
        }
        if let Some(running) = state.running.get_value() {
            if let Some(storage) = session_storage() {
                let _ = storage.set_item(UPDATED_FROM_KEY, &running);
            }
        }
        if let Some(window) = web_sys::window() {
            if window.location().reload().is_err() {
                state.prompt.set(BuildPrompt::Failed);
            }
        }
    });
}

fn check(state: BuildVersionState, last_check: StoredValue<Option<f64>>, trigger: CheckTrigger) {
    let now = super::now_ms();
    if !should_check(trigger, last_check.get_value(), now, page_visible()) {
        return;
    }
    last_check.set_value(Some(now));
    spawn_local(async move {
        let Some(manifest) = fetch_manifest().await else {
            return;
        };
        let running = state.running.get_value();
        if running.as_deref() == Some(manifest.build.as_str()) {
            return;
        }
        let known = state
            .latest
            .with_value(|latest| latest.as_ref().map(|m| m.build == manifest.build))
            .unwrap_or(false);
        if !known {
            state.latest.set_value(Some(manifest));
            state.prefetched.set_value(false);
        }

        let prompt = state.prompt.get_untracked();
        if prompt == BuildPrompt::Switching {
            return;
        }
        if trigger == CheckTrigger::ServerFnError {
            state.prompt.set(BuildPrompt::ReloadDialog);
            state.dialog_open.set(true);
        }
        if known {
            return;
        }
        let ok = prefetch_latest(state).await;
        state.prefetched.set_value(ok);
        // The bar shows even when the prefetch failed; Reload now retries it.
        if state.prompt.get_untracked() == BuildPrompt::Hidden {
            state.prompt.set(BuildPrompt::Available);
        }
    });
}

fn running_build_from_document() -> Option<String> {
    let document = web_sys::window()?.document()?;
    let url = ["link[rel=modulepreload]", "script[type=module][src]"]
        .iter()
        .find_map(|selector| {
            let el = document.query_selector(selector).ok()??;
            el.get_attribute("href").or_else(|| el.get_attribute("src"))
        })?;
    match parse_build_id(&url) {
        RunningBuild::Stamped(id) => Some(id),
        RunningBuild::Unstamped | RunningBuild::Missing => None,
    }
}

fn show_updated_toast(toaster: Option<ToasterInjection>, running: &str) {
    let Some(storage) = session_storage() else {
        return;
    };
    let Ok(Some(from)) = storage.get_item(UPDATED_FROM_KEY) else {
        return;
    };
    let _ = storage.remove_item(UPDATED_FROM_KEY);
    if from != running {
        if let Some(toaster) = toaster {
            toaster.dispatch(
                ToastOptions::new("Updated to the latest version.")
                    .intent(ToastIntent::Success.into())
                    .timeout(UPDATED_TOAST_MS),
            );
        }
    }
}

fn session_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.session_storage().ok()?
}

fn page_visible() -> bool {
    web_sys::window()
        .and_then(|w| w.document())
        .is_some_and(|d| d.visibility_state() == VisibilityState::Visible)
}

fn register_service_worker() {
    let Some(window) = web_sys::window() else {
        return;
    };
    let navigator = window.navigator();
    // Insecure origins and some private modes have no service workers.
    if !js_sys::Reflect::has(&navigator, &JsValue::from_str("serviceWorker")).unwrap_or(false) {
        return;
    }
    let options = RegistrationOptions::new();
    options.set_scope("/");
    let promise = navigator
        .service_worker()
        .register_with_options(SERVICE_WORKER_PATH, &options);
    spawn_local(async move {
        let _ = JsFuture::from(promise).await;
    });
}

async fn fetch_manifest() -> Option<BuildManifest> {
    let window = web_sys::window()?;
    let init = RequestInit::new();
    init.set_cache(RequestCache::NoStore);
    let response = JsFuture::from(window.fetch_with_str_and_init(BUILD_JSON_PATH, &init))
        .await
        .ok()?
        .dyn_into::<Response>()
        .ok()?;
    // A 404 means the host doesn't serve version info: stay quiet.
    if !response.ok() {
        return None;
    }
    let text = JsFuture::from(response.text().ok()?)
        .await
        .ok()?
        .as_string()?;
    let manifest: BuildManifest = serde_json::from_str(&text).ok()?;
    super::is_valid_build_id(&manifest.build).then_some(manifest)
}

/// Ask the service worker to cache the latest build. `true` once it confirms.
async fn prefetch_latest(state: BuildVersionState) -> bool {
    let Some(manifest) = state.latest.get_value() else {
        return false;
    };
    let running = state.running.get_value();
    prefetch(&manifest, running.as_deref())
        .await
        .unwrap_or(false)
}

async fn prefetch(manifest: &BuildManifest, running: Option<&str>) -> Option<bool> {
    let window = web_sys::window()?;
    let navigator = window.navigator();
    if !js_sys::Reflect::has(&navigator, &JsValue::from_str("serviceWorker")).ok()? {
        return Some(false);
    }
    let ready = navigator.service_worker().ready().ok()?;
    let registration = with_timeout(ready, READY_TIMEOUT_MS)
        .await?
        .dyn_into::<ServiceWorkerRegistration>()
        .ok()?;
    let worker = registration.active()?;

    let message = serde_json::json!({
        "type": "prefetch",
        "build": manifest.build,
        "running": running,
        "assets": manifest.assets,
    });
    let message = js_sys::JSON::parse(&message.to_string()).ok()?;

    let channel = MessageChannel::new().ok()?;
    let port = channel.port1();
    let reply = js_sys::Promise::new(&mut |resolve, _reject| {
        let on_message = Closure::once_into_js(move |event: MessageEvent| {
            let _ = resolve.call1(&JsValue::NULL, &event.data());
        });
        port.set_onmessage(Some(on_message.unchecked_ref()));
    });
    worker
        .post_message_with_transferable(&message, &js_sys::Array::of1(&channel.port2()))
        .ok()?;
    let data = with_timeout(reply, PREFETCH_TIMEOUT_MS).await?;
    port.close();
    let kind = js_sys::Reflect::get(&data, &JsValue::from_str("type"))
        .ok()?
        .as_string()?;
    Some(kind == "prefetch-done")
}

/// Await `promise`, or give up after `ms`. `None` on rejection or timeout.
async fn with_timeout(promise: js_sys::Promise, ms: i32) -> Option<JsValue> {
    let window = web_sys::window()?;
    let timeout = js_sys::Promise::new(&mut |resolve, _reject| {
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms);
    });
    let value = JsFuture::from(js_sys::Promise::race(&js_sys::Array::of2(
        &promise, &timeout,
    )))
    .await
    .ok()?;
    (!value.is_undefined()).then_some(value)
}
