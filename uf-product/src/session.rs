//! Client-side auth session loading helpers.
//!
//! Maps the host axum-login / tower-sessions session into [`AuthSession`] for
//! reactive UI state. Call [`init_auth_resource`] from the host app root after
//! [`provide_auth_context`]. UI code should read profiles with
//! [`crate::use_authenticated_user`] (display name, email, roles), not treat
//! `AuthSession::is_authenticated()` as a user label.
//!
//! # Example
//!
//! ```rust,ignore
//! use leptos::prelude::*;
//! use uf_product::{init_auth_resource, provide_auth_context, use_authenticated_user};
//!
//! #[component]
//! fn AppRoot() -> impl IntoView {
//!     let auth = provide_auth_context(Default::default());
//!     let _session = init_auth_resource(&auth);
//!     let user = use_authenticated_user();
//!     view! {
//!         <p>{move || user.get().and_then(|u| u.display_name.clone()).unwrap_or_default()}</p>
//!     }
//! }
//! ```

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
#[cfg(target_arch = "wasm32")]
use std::time::Duration;

use leptos::logging::warn;
use leptos::prelude::*;

#[cfg(feature = "ssr")]
use crate::{AnonymousUser, AuthenticatedUser};
use crate::{AuthContext, AuthSession};

/// Fetch the current authentication session from the server.
///
/// # Errors
///
/// Returns [`ServerFnError`] when the SSR extractor fails to obtain the auth
/// session (misconfigured host middleware). Anonymous or signed-out users return
/// [`Ok`] with [`AuthSession::Anonymous`] or the default session.
#[server(GetSession)]
pub async fn get_session() -> Result<AuthSession, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use leptos_axum::extract;

        let auth_session: axum_login::AuthSession<lepton_host_adapter::Backend> = extract().await?;

        if let Some(user) = auth_session.user.clone() {
            let profile = AuthenticatedUser {
                user_id: user.id.to_string(),
                email: Some(user.email.clone()),
                display_name: user.display_name.clone(),
                avatar_url: None,
                roles: user.roles.clone(),
                email_verified: user.email_verified,
            };
            return Ok(AuthSession::Authenticated(profile));
        }

        if let Ok(session) = extract::<tower_sessions::Session>().await {
            if let Ok(Some(reason)) = session.get::<String>("auth_reason").await {
                return Ok(AuthSession::Anonymous(AnonymousUser {
                    reason: Some(reason),
                }));
            }
        }
    }

    Ok(AuthSession::default())
}

/// Fail-closed policy when [`get_session`] returns an error: keep the prior
/// session instead of inventing an authenticated user.
fn apply_session_load_result(
    prior: AuthSession,
    result: Result<AuthSession, ServerFnError>,
) -> AuthSession {
    match result {
        Ok(session) => session,
        Err(_) => prior,
    }
}

fn apply_loaded_session(
    session_signal: RwSignal<AuthSession>,
    auth: &AuthContext,
    result: Result<AuthSession, ServerFnError>,
) {
    let prior = session_signal.get_untracked();
    if let Err(err) = &result {
        warn!("Failed to load auth session: {err}");
    }
    session_signal.set(apply_session_load_result(prior, result));
    auth.mark_session_loaded();
}

/// True when a deferred hydrate apply still matches the latest `get_session` fetch.
///
/// A stale anonymous apply scheduled during hydrate must not overwrite the
/// authenticated session from a later sign-in refresh.
#[must_use]
fn session_apply_is_current(scheduled: u32, latest: u32) -> bool {
    scheduled == latest
}

/// Create a resource that keeps [`AuthContext`] in sync with the backend session.
///
/// SSR renders from the default (anonymous) session signal — Effects do not run
/// on the server. On the client, the serialized [`get_session`] payload is applied
/// after the hydrate cursor finishes, via `setTimeout`, so `hydrate_lazy` can
/// complete instead of starving in a microtask loop.
///
/// # Examples
///
/// ```rust,ignore
/// use uf_product::{init_auth_resource, provide_auth_context};
///
/// let auth = provide_auth_context(Default::default());
/// let _auth_resource = init_auth_resource(&auth);
/// ```
pub fn init_auth_resource(auth: &AuthContext) -> Resource<Result<AuthSession, ServerFnError>> {
    let reload_token = auth.reload_token();
    let auth_resource = Resource::new(move || reload_token.get(), |_| get_session());

    let session_signal = auth.session();
    let auth_for_loaded = auth.clone();
    let apply_generation = Arc::new(AtomicU32::new(0));
    Effect::new(move |_| {
        let Some(result) = auth_resource.get() else {
            return;
        };
        let generation = apply_generation.fetch_add(1, Ordering::Relaxed) + 1;

        #[cfg(target_arch = "wasm32")]
        {
            let hydration = Owner::current_shared_context();
            let still_hydrating =
                move || hydration.as_ref().is_some_and(|sc| sc.during_hydration());
            let session_signal = session_signal;
            let auth_for_loaded = auth_for_loaded.clone();
            let apply_generation = Arc::clone(&apply_generation);
            defer_until_hydration_completes(still_hydrating, 0, move || {
                if !session_apply_is_current(generation, apply_generation.load(Ordering::Relaxed)) {
                    return;
                }
                apply_loaded_session(session_signal, &auth_for_loaded, result);
            });
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            if session_apply_is_current(generation, apply_generation.load(Ordering::Relaxed)) {
                apply_loaded_session(session_signal, &auth_for_loaded, result);
            }
        }
    });

    auth_resource
}

/// Capture the hydrate flag under the root owner. `setTimeout` runs outside that
/// owner; polling the captured flag lets `hydrate_lazy` finish first.
#[cfg(target_arch = "wasm32")]
fn defer_until_hydration_completes(
    still_hydrating: impl Fn() -> bool + 'static,
    tries: u8,
    apply: impl FnOnce() + 'static,
) {
    const MAX_TRIES: u8 = 250;
    if !still_hydrating() || tries >= MAX_TRIES {
        apply();
        return;
    }
    set_timeout(
        move || defer_until_hydration_completes(still_hydrating, tries.saturating_add(1), apply),
        Duration::from_millis(16),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_bridge_anonymous_default_happy_path() {
        let session = AuthSession::default();
        assert!(matches!(session, AuthSession::Anonymous(_)));
    }

    #[test]
    fn session_bridge_authenticated_maps_profile_happy_path() {
        let profile = AuthenticatedUser {
            user_id: "u-1".into(),
            email: Some("a@b.c".into()),
            display_name: Some("Ada".into()),
            avatar_url: None,
            roles: vec!["user".into()],
            email_verified: true,
        };
        let session = AuthSession::Authenticated(profile.clone());
        match session {
            AuthSession::Authenticated(u) => {
                assert_eq!(u.user_id, "u-1");
                assert_eq!(u.email.as_deref(), Some("a@b.c"));
                assert_eq!(u.display_name.as_deref(), Some("Ada"));
                assert!(u.email_verified);
            }
            AuthSession::Anonymous(_) => panic!("expected authenticated"),
        }
    }

    #[test]
    fn session_bridge_extract_failure_sad() {
        let prior = AuthSession::Anonymous(AnonymousUser {
            reason: Some("signed_out".into()),
        });
        let err = ServerFnError::new("extractor unavailable");
        let applied = apply_session_load_result(prior.clone(), Err(err));
        assert!(
            matches!(
                &applied,
                AuthSession::Anonymous(u) if u.reason.as_deref() == Some("signed_out")
            ),
            "extract failure must keep prior anonymous session"
        );
        assert!(
            !matches!(applied, AuthSession::Authenticated(_)),
            "extract failure must not elevate to Authenticated"
        );
    }

    #[test]
    fn stale_deferred_session_apply_is_dropped_sad() {
        assert!(
            session_apply_is_current(2, 2),
            "the latest get_session generation must still apply"
        );
        assert!(
            !session_apply_is_current(1, 2),
            "hydrate-deferred anonymous apply must not overwrite a newer sign-in fetch"
        );
    }
}
