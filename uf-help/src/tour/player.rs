//! Spotlight tour player mounted by the product shell.

use leptos::prelude::*;
use leptos_router::hooks::use_location;
use uf_product::primitives::{
    PopoverPosition, SpotlightBody, SpotlightHeader, SpotlightTour, SpotlightTourStep,
};
use uf_product::{use_access_gate_active, use_auth_state, AuthSession};

use super::replay_bus::install_help_replay_bus;
use crate::server::{help_list_visits_for_route, help_mark_steps_seen};
use crate::service::{
    compute_pending, local_mark_steps_seen, read_local_visits, read_local_visits_for_route,
    HelpStepKey, HelpVisitRecord,
};

/// Drives Orbital [`SpotlightTour`] for pending help steps on the current route.
///
/// Pending = inventory step with no visit, or visit with `replay == true`. New
/// `feature_highlight` keys show automatically for returning users. Auto-play
/// is skipped while [`uf_product::AccessGateActive`] is set (sign-in, email
/// verification, and permission-required empty states).
#[allow(clippy::unit_arg)]
#[component]
pub fn HelpTourPlayer() -> impl IntoView {
    let location = use_location();
    let auth = use_auth_state();
    let open = RwSignal::new(false);
    let reload = RwSignal::new(0u32);
    let pending_keys = RwSignal::new(Vec::<HelpStepKey>::new());
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

    Effect::new(move |_| {
        let pathname = location.pathname.get();
        let visits = resolve_visits();
        let inventory = crate::collect_help_steps_for_route(&pathname);
        let pending = compute_pending(&inventory, &visits);
        let keys: Vec<HelpStepKey> = pending
            .iter()
            .map(|d| HelpStepKey {
                route: d.route.to_string(),
                feature_highlight: d.feature_highlight.to_string(),
                spotlight: d.spotlight.map(str::to_string),
            })
            .collect();
        pending_keys.set(keys);
        let gated = access_gate.is_some_and(|g| g.get());
        open.set(client_ready.get() && !pending.is_empty() && !gated);
    });

    let on_finish = Callback::new(move |_| {
        let keys = pending_keys.get_untracked();
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
                let pathname = location.pathname.get();
                let visits = resolve_visits();
                let inventory = crate::collect_help_steps_for_route(&pathname);
                let pending = compute_pending(&inventory, &visits);
                let gated = access_gate.is_some_and(|g| g.get());
                if !client_ready.get() || pending.is_empty() || gated {
                    return view! { <></> }.into_any();
                }
                view! {
                    <SpotlightTour open=open on_finish=on_finish>
                        {pending
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
