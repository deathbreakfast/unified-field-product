//! Valence privacy evaluators for welcome featured-app schemas.
//!
//! Uses [`gauge::actor_can_raw`] so policy checks do not re-enter typed ORM privacy.

use async_trait::async_trait;
use std::any::Any;
use valence::{Actor, ActorContext, Error, PolicyEvaluator, PrivacyOperation, Result, Valence};

/// Gauge permission name for featured catalog CUD.
pub const WELCOME_ADMIN_PERMISSION: &str = "WelcomeAdmin";

/// Session actors holding `WelcomeAdmin` may mutate featured catalog rows.
pub const WELCOME_ADMIN_GATE: WelcomeAdminGate = WelcomeAdminGate;

/// Static WelcomeAdmin gate (raw Gauge walks).
#[derive(Debug, Clone, Copy)]
pub struct WelcomeAdminGate;

#[async_trait]
impl PolicyEvaluator for WelcomeAdminGate {
    fn name(&self) -> &'static str {
        "welcome::WELCOME_ADMIN"
    }

    fn description(&self) -> Option<&'static str> {
        Some("WelcomeAdmin Gauge permission (raw walks)")
    }

    async fn evaluate(
        &self,
        _op: PrivacyOperation,
        _record: &serde_json::Value,
        actor: &dyn ActorContext,
        v: &Valence,
    ) -> Result<bool> {
        let viewer: Actor = serde_json::from_value(actor.actor_json().clone())
            .map_err(|e| Error::Internal(format!("invalid actor context: {e}")))?;
        if viewer.is_system() {
            return Ok(true);
        }
        gauge::actor_can_raw::actor_can_raw(v, WELCOME_ADMIN_PERMISSION)
            .await
            .map_err(|e| Error::Privacy(format!("WelcomeAdmin raw check failed: {e}")))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
