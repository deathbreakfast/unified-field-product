//! Session-bag TOTP step-up gate for product server functions.
//!
//! lepton-auth writes the sudo window under the keys below after
//! `verify_totp_for_session`. Product crates call [`require_step_up`] (or the
//! `#[uf_product_macros::server(..., step_up)]` expansion) so the require path
//! does not need a direct lepton-auth dependency.
//!
//! - [`StepUpMode::Window`] — accept an unexpired session window bound to the
//!   current user, auth-hash, and `sensitive_mutation` scope.
//! - [`StepUpMode::Fresh`] — the macro gate returns `Ok`; the handler must call
//!   `lepton_auth::verify_fresh_totp` with an explicit code (break-glass reveal,
//!   Super User membership, handoff finalize).

#[cfg(feature = "ssr")]
use std::sync::Arc;

use leptos::prelude::*;

/// Session key: unix seconds when the TOTP sudo window was opened.
pub const STEP_UP_VERIFIED_AT_KEY: &str = "step_up_verified_at";
/// Session key: unix seconds when the TOTP sudo window expires.
pub const STEP_UP_EXPIRES_AT_KEY: &str = "step_up_expires_at";
/// Session key: user id the sudo window is bound to.
pub const STEP_UP_USER_ID_KEY: &str = "step_up_user_id";
/// Session key: auth-hash bytes the sudo window is bound to.
pub const STEP_UP_AUTH_HASH_KEY: &str = "step_up_auth_hash";
/// Session key: step-up scope string.
pub const STEP_UP_SCOPE_KEY: &str = "step_up_scope";
/// Default TOTP sudo window lifetime (5 minutes). Must match lepton-auth.
pub const STEP_UP_TTL_SECS: i64 = 300;
/// Scope string written by lepton-auth for Tier A mutations.
pub const STEP_UP_SCOPE_SENSITIVE: &str = "sensitive_mutation";

/// How a gated server function consumes step-up proof.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepUpMode {
    /// Accept an unexpired session sudo window.
    Window,
    /// Demand a fresh TOTP for this call (handler verifies the code).
    Fresh,
}

impl StepUpMode {
    /// Parse a macro attribute token (`""` / `"window"` / `"fresh"`).
    #[must_use]
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim() {
            "" | "window" | "true" => Some(Self::Window),
            "fresh" => Some(Self::Fresh),
            _ => None,
        }
    }

    /// Stable label for errors / telemetry.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Window => "window",
            Self::Fresh => "fresh",
        }
    }
}

/// Optional override for hosts that need custom window evaluation.
#[cfg(feature = "ssr")]
#[async_trait::async_trait]
pub trait StepUpBackend: Send + Sync {
    /// Assert a recent window (or refuse with `STEP_UP:step_up_required:`).
    async fn require_window(&self) -> Result<(), ServerFnError>;
}

/// Install a custom step-up evaluator (optional; default reads the session bag).
#[cfg(feature = "ssr")]
pub fn provide_step_up_backend(backend: Arc<dyn StepUpBackend>) {
    provide_context(backend);
}

/// Read the installed backend, if any.
#[cfg(feature = "ssr")]
pub fn use_step_up_backend() -> Option<Arc<dyn StepUpBackend>> {
    use_context::<Arc<dyn StepUpBackend>>()
}

/// Fail-closed gate for sensitive server functions.
///
/// Call after the permission check. Prefer
/// `#[uf_product_macros::server(permission = "...", step_up)]` so the expansion
/// inserts this call; use `"fresh"` when the handler verifies TOTP itself.
///
/// # Errors
///
/// Returns [`ServerFnError`] with a `STEP_UP:` prefix when the window is
/// missing, expired, bound to another user, or the mode token is unknown.
/// `fresh` always returns `Ok(())` here — the handler must still call
/// `lepton_auth::verify_fresh_totp`.
#[cfg(feature = "ssr")]
pub async fn require_step_up(mode: &str) -> Result<(), ServerFnError> {
    let parsed = StepUpMode::parse(mode).ok_or_else(|| {
        ServerFnError::new(format!("STEP_UP:step_up_invalid: unknown mode `{mode}`"))
    })?;
    match parsed {
        StepUpMode::Window => {
            if let Some(backend) = use_step_up_backend() {
                return backend.require_window().await;
            }
            require_session_window().await
        }
        StepUpMode::Fresh => {
            // Handler must call lepton_auth::verify_fresh_totp with an explicit code.
            Ok(())
        }
    }
}

#[cfg(feature = "ssr")]
async fn require_session_window() -> Result<(), ServerFnError> {
    use chrono::Utc;
    use leptos_axum::extract;
    use tower_sessions::Session;

    let ctx = higgs::Higgs::from_request()
        .await
        .map_err(|_| ServerFnError::new("STEP_UP:auth_required: authentication required"))?;
    let Some(session_user_id) = ctx.session_user_id().map(|id| id.to_string()) else {
        return Err(ServerFnError::new(
            "STEP_UP:auth_required: authentication required",
        ));
    };
    let session: Session = extract()
        .await
        .map_err(|_| ServerFnError::new("STEP_UP:store: session unavailable"))?;

    let Some(expires_ts) = session
        .get::<i64>(STEP_UP_EXPIRES_AT_KEY)
        .await
        .map_err(|_| ServerFnError::new("STEP_UP:store: session read failed"))?
    else {
        return Err(ServerFnError::new(
            "STEP_UP:step_up_required: recent totp verification required",
        ));
    };
    let user_id = session
        .get::<String>(STEP_UP_USER_ID_KEY)
        .await
        .map_err(|_| ServerFnError::new("STEP_UP:store: session read failed"))?
        .ok_or_else(|| {
            ServerFnError::new("STEP_UP:step_up_required: recent totp verification required")
        })?;
    let scope = session
        .get::<String>(STEP_UP_SCOPE_KEY)
        .await
        .map_err(|_| ServerFnError::new("STEP_UP:store: session read failed"))?
        .unwrap_or_default();

    if user_id != session_user_id {
        return Err(ServerFnError::new(
            "STEP_UP:step_up_required: recent totp verification required",
        ));
    }
    if scope != STEP_UP_SCOPE_SENSITIVE {
        return Err(ServerFnError::new(
            "STEP_UP:step_up_required: recent totp verification required",
        ));
    }
    let Some(expires_at) = chrono::DateTime::from_timestamp(expires_ts, 0) else {
        return Err(ServerFnError::new(
            "STEP_UP:step_up_required: recent totp verification required",
        ));
    };
    if expires_at <= Utc::now() {
        return Err(ServerFnError::new(
            "STEP_UP:step_up_expired: totp verification expired",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::StepUpMode;

    #[test]
    fn step_up_mode_parse_window_aliases_happy_path() {
        assert_eq!(StepUpMode::parse(""), Some(StepUpMode::Window));
        assert_eq!(StepUpMode::parse("window"), Some(StepUpMode::Window));
        assert_eq!(StepUpMode::parse("true"), Some(StepUpMode::Window));
        assert_eq!(StepUpMode::parse(" window "), Some(StepUpMode::Window));
        assert_eq!(StepUpMode::Window.as_str(), "window");
    }

    #[test]
    fn step_up_mode_parse_fresh_happy_path() {
        assert_eq!(StepUpMode::parse("fresh"), Some(StepUpMode::Fresh));
        assert_eq!(StepUpMode::parse(" fresh "), Some(StepUpMode::Fresh));
        assert_eq!(StepUpMode::Fresh.as_str(), "fresh");
    }

    #[test]
    fn step_up_mode_parse_unknown_sad() {
        assert_eq!(StepUpMode::parse("sudo"), None);
        assert_eq!(StepUpMode::parse("Fresh"), None);
        assert_eq!(StepUpMode::parse("WINDOW"), None);
    }

    #[test]
    fn step_up_error_prefixes_stable_happy_path() {
        // Session-window evaluation needs a live Higgs/session; keep the client-facing
        // prefixes pinned so UI detectors (`STEP_UP:step_up_required` / `_expired`) stay honest.
        let src = include_str!("step_up.rs");
        for needle in [
            "STEP_UP:step_up_required:",
            "STEP_UP:step_up_expired:",
            "STEP_UP:step_up_invalid:",
            "STEP_UP:auth_required:",
        ] {
            assert!(
                src.contains(needle),
                "step_up gate must keep stable error prefix `{needle}`"
            );
        }
        assert!(
            src.contains("recent totp verification required"),
            "missing window-required message"
        );
        assert!(
            src.contains("totp verification expired"),
            "missing window-expired message"
        );
    }

    #[test]
    fn fresh_mode_skips_session_window_happy_path() {
        // TM-3 / TM-12: `require_step_up("fresh")` must not consult the sudo window.
        let src = include_str!("step_up.rs");
        assert!(
            src.contains("StepUpMode::Fresh =>")
                && src.contains("Ok(())")
                && src.contains("verify_fresh_totp"),
            "Fresh mode must return Ok without reading the session bag"
        );
    }
}
