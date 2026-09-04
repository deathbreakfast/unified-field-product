//! Server function for AppBar workspace content search.

use leptos::prelude::*;

use super::WorkspaceSearchHit;

/// Query the signed-in user's workspace content index.
///
/// Uses session Valence only — never `system_valence()`.
///
/// # Errors
///
/// Returns [`ServerFnError`] when the caller is not authenticated (`"Not authenticated"`),
/// session Valence cannot be built, [`super::query`] fails (message is the
/// [`super::WorkspaceSearchError`] display string: unauthenticated, invalid actor,
/// invalid query, or Valence failure), or the `ssr` feature is off
/// (`"query_workspace_search requires the ssr feature"`).
#[server(QueryWorkspaceSearch)]
pub async fn query_workspace_search(
    query: String,
    max: u32,
) -> Result<Vec<WorkspaceSearchHit>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let valence = resolve_user_valence().await?;
        crate::workspace_search::query(&valence, &query, max)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = (query, max);
        Err(ServerFnError::new(
            "query_workspace_search requires the ssr feature",
        ))
    }
}

#[cfg(feature = "ssr")]
async fn resolve_user_valence() -> Result<valence::Valence, ServerFnError> {
    match crate::ssr::higgs().await {
        Ok(ctx) => {
            if ctx.session_user_id().is_none() {
                return Err(ServerFnError::new("Not authenticated"));
            }
            crate::ssr::valence(&ctx)
        }
        Err(_) => harness_session_valence().await,
    }
}

/// Playwright / shell harness: `Arc<Valence>` context + tower-sessions `e2e_auth_kind`.
#[cfg(feature = "ssr")]
async fn harness_session_valence() -> Result<valence::Valence, ServerFnError> {
    use leptos_axum::extract;
    use std::sync::Arc;
    use tower_sessions::Session;
    use valence::Actor;

    let base = use_context::<Arc<valence::Valence>>()
        .ok_or_else(|| ServerFnError::new("Not authenticated"))?;
    let session: Session = extract().await?;
    let kind = session
        .get::<String>("e2e_auth_kind")
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .unwrap_or_else(|| "anonymous".into());
    let actor = match kind.as_str() {
        "authenticated_verified" => Actor::User {
            user_id: "e2e-user".into(),
        },
        "authenticated_unverified" => Actor::User {
            user_id: "e2e-unverified".into(),
        },
        _ => return Err(ServerFnError::new("Not authenticated")),
    };
    Ok(base.with_actor(actor))
}
