//! Spotlight tour player mounted by the product shell.

use leptos::prelude::*;
use leptos_router::hooks::use_location;
use uf_product::primitives::{
    PopoverPosition, SpotlightBody, SpotlightHeader, SpotlightTour, SpotlightTourStep,
};
use uf_product::{use_access_gate_active, use_auth_state, AuthSession};

use std::time::Duration;

use super::anchor_probe::anchor_visible;
use super::replay_bus::install_help_replay_bus;
use crate::server::{help_list_visits_for_route, help_mark_steps_seen};
use crate::service::presentable::{presentable_steps, presented_keys, should_open};
use crate::service::{
    compute_pending, local_mark_steps_seen, read_local_visits, read_local_visits_for_route,
    HelpStepKey, HelpVisitRecord,
};
use crate::HelpStepDescriptor;

/// How often the player re-checks the DOM for anchors of waiting steps.
const ANCHOR_POLL: Duration = Duration::from_millis(400);

/// Drives Orbital [`SpotlightTour`] for pending help steps on the current route.
///
/// Pending = inventory step with no visit, or visit with `replay == true`. New
/// `feature_highlight` keys show automatically for returning users. Auto-play
/// is skipped while [`uf_product::AccessGateActive`] is set (sign-in, email
/// verification, and permission-required empty states).
///
/// A step with a `spotlight` id waits until that element is on screen. The
/// player polls while steps are waiting and opens once the showable set is the
/// same on two consecutive checks. Only the steps a tour showed are marked
/// seen; the rest stay pending for when their UI appears.
#[allow(clippy::unit_arg)]
#[component]
pub fn HelpTourPlayer() -> impl IntoView {
    let location = use_location();
    let auth = use_auth_state();
    let open = RwSignal::new(false);
    let reload = RwSignal::new(0u32);
    let presented = RwSignal::new(Vec::<&'static HelpStepDescriptor>::new());
    let last_poll = StoredValue::new(Vec::<HelpStepKey>::new());
    let has_pending = RwSignal::new(false);
    let poll_tick = RwSignal::new(0u32);
    let poll_handle = StoredValue::new(None::<IntervalHandle>);
    let replay_tick = RwSignal::new(0u32);
    // Start closed on both SSR and first hydrate paint, then open on the client
    // after ownership is live. Avoids Backdrop hydration mismatches.
    let client_ready = RwSignal::new(false);
    let access_gate = use_access_gate_active();
    install_help_replay_bus(replay_tick);

    Effect::new(move |_| {
        if cfg!(target_arch = "wasm32") {
            client_ready.set(true);
        }
    });

    // Server visits for signed-in users. Anon always uses localStorage on the
    // client — do not trust SSR-serialized empty visits after hydrate.
    let server_visits = Resource::new(
        move || {
            (
                location.pathname.get(),
                reload.get(),
                replay_tick.get(),
                auth.get(),
            )
        },
        |(pathname, _, _, session)| async move {
            if matches!(session, AuthSession::Authenticated(_)) {
                help_list_visits_for_route(pathname).await.ok()
            } else {
                None
            }
        },
    );

    let resolve_visits = move || -> Vec<HelpVisitRecord> {
        let pathname = location.pathname.get();
        let _ = reload.get();
        let _ = replay_tick.get();
        let session = auth.get();
        let local = read_local_visits_for_route(&pathname);
        if matches!(session, AuthSession::Authenticated(_)) {
            match server_visits.get() {
                Some(Some(server)) => {
                    // Server rows win on conflict; keep local-only rows so a
                    // route-scoped / in-flight Valence list cannot drop device progress.
                    crate::service::merge_local_into_server(&server, &local)
                }
                Some(None) | None => local,
            }
        } else {
            local
        }
    };

    // Leaving a route closes its tour without marking anything seen.
    Effect::new(move |previous: Option<String>| {
        let pathname = location.pathname.get();
        if previous.is_some_and(|p| p != pathname) {
            open.set(false);
            presented.set(Vec::new());
            last_poll.set_value(Vec::new());
        }
        pathname
    });

    Effect::new(move |_| {
        let _ = poll_tick.get();
        let pathname = location.pathname.get();
        let visits = resolve_visits();
        let ready = client_ready.get();
        let gated = access_gate.is_some_and(|g| g.get());
        if open.get() {
            return;
        }
        let inventory = crate::collect_help_steps_for_route(&pathname);
        let pending = compute_pending(&inventory, &visits);
        has_pending.set(!pending.is_empty());
        let showable = presentable_steps(&pending, anchor_visible);
        let keys = presented_keys(&showable);
        let previous = last_poll.get_value();
        if should_open(&previous, &keys, ready, gated) {
            last_poll.set_value(Vec::new());
            presented.set(showable);
            open.set(true);
        } else {
            last_poll.set_value(keys);
        }
    });

    // Poll only while steps are waiting and no tour is open.
    Effect::new(move |_| {
        let gated = access_gate.is_some_and(|g| g.get());
        let want = client_ready.get() && has_pending.get() && !open.get() && !gated;
        let running = poll_handle.with_value(Option::is_some);
        if want && !running {
            let tick = move || {
                let _ = poll_tick.try_update(|n| *n = n.wrapping_add(1));
            };
            if let Ok(handle) = set_interval_with_handle(tick, ANCHOR_POLL) {
                poll_handle.set_value(Some(handle));
            }
        } else if !want && running {
            if let Some(Some(handle)) = poll_handle.try_update_value(Option::take) {
                handle.clear();
            }
        }
    });
    on_cleanup(move || {
        if let Some(Some(handle)) = poll_handle.try_update_value(Option::take) {
            handle.clear();
        }
    });

    let on_finish = Callback::new(move |_| {
        let keys = presented_keys(&presented.get_untracked());
        if keys.is_empty() {
            open.set(false);
            return;
        }
        // Always mirror to localStorage so hydrate after SSR does not resurrect
        // a completed tour when Higgs/Valence is unavailable (lab / e2e).
        local_mark_steps_seen(&keys);
        open.set(false);
        let authed = matches!(auth.get_untracked(), AuthSession::Authenticated(_));
        let local = read_local_visits();
        // Defer reload so we do not unmount SpotlightTour (and its Finish button)
        // while still inside the dismiss/click stack — that panics on disposed
        // signals and can take down the e2e SSR process (ERR_EMPTY_RESPONSE).
        leptos::task::spawn_local(async move {
            presented.set(Vec::new());
            reload.update(|n| *n = n.wrapping_add(1));
            if authed {
                let _ = help_mark_steps_seen(keys, local).await;
                reload.update(|n| *n = n.wrapping_add(1));
            }
        });
    });

    view! {
        <div data-testid="help-tour-player">
            {move || {
                let steps = presented.get();
                let gated = access_gate.is_some_and(|g| g.get());
                if !client_ready.get() || steps.is_empty() || gated {
                    return view! { <></> }.into_any();
                }
                view! {
                    <SpotlightTour open=open on_finish=on_finish>
                        {steps
                            .into_iter()
                            .map(|d| {
                                let title = d.title.to_string();
                                let render = d.render;
                                let anchor = d.spotlight.map(str::to_string).unwrap_or_default();
                                let position = d.position.unwrap_or(PopoverPosition::Top);
                                view! {
                                    <SpotlightTourStep anchor_id=anchor position=position>
                                        <SpotlightHeader slot>{title}</SpotlightHeader>
                                        <SpotlightBody slot>{(render)()}</SpotlightBody>
                                    </SpotlightTourStep>
                                }
                            })
                            .collect_view()}
                    </SpotlightTour>
                }
                .into_any()
            }}
        </div>
    }
}

/// Request replay for the current route (Valence when signed in, else localStorage).
pub fn request_replay_current_route(route: String) {
    use super::replay_bus::notify_help_replay;
    use crate::server::help_request_replay_for_route;
    use crate::service::local_request_replay_for_route;
    use uf_product::{use_auth_state, AuthSession};

    // Always mirror replay into localStorage so route-scoped replay works without Higgs.
    local_request_replay_for_route(&route);
    let auth = use_auth_state();
    let authed = matches!(auth.get_untracked(), AuthSession::Authenticated(_));
    if authed {
        leptos::task::spawn_local(async move {
            let _ = help_request_replay_for_route(route).await;
            notify_help_replay();
        });
    } else {
        notify_help_replay();
    }
}
