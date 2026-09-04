//! Owner-scoped workspace search query.

use valence::{Actor, Valence};

use super::{WorkspaceSearchError, WorkspaceSearchHit};

const MAX_QUERY_CHARS: usize = 200;
const DEFAULT_MAX_RESULTS: u32 = 20;
const ABSOLUTE_MAX_RESULTS: u32 = 50;

/// Search the caller's index documents.
///
/// Requires a [`Actor::User`] on `valence`. Anonymous and System actors are rejected
/// ([`WorkspaceSearchError::Unauthenticated`] / [`WorkspaceSearchError::InvalidActor`]).
///
/// # Errors
///
/// Returns [`WorkspaceSearchError::Unauthenticated`] for anonymous actors,
/// [`WorkspaceSearchError::InvalidActor`] for System / ServiceUser actors,
/// [`WorkspaceSearchError::InvalidQuery`] when the query text fails validation
/// (empty, too long, or otherwise rejected), or [`WorkspaceSearchError::Valence`]
/// when the index read fails.
///
/// # Examples
///
/// ```rust,ignore
/// use uf_product::workspace_search::query;
///
/// # async fn demo(v: &valence::Valence) -> Result<(), uf_product::workspace_search::WorkspaceSearchError> {
/// let hits = query(v, "beacon", 10).await?;
/// assert!(hits.iter().all(|h| h.link.starts_with('/')));
/// # Ok(())
/// # }
/// ```
pub async fn query(
    valence: &Valence,
    query_text: &str,
    max_results: u32,
) -> Result<Vec<WorkspaceSearchHit>, WorkspaceSearchError> {
    match valence.actor() {
        Actor::Anonymous => return Err(WorkspaceSearchError::Unauthenticated),
        Actor::System { .. } => {
            return Err(WorkspaceSearchError::InvalidActor {
                reason: "system_actor_forbidden",
            })
        }
        Actor::ServiceUser { .. } => {
            return Err(WorkspaceSearchError::InvalidActor {
                reason: "service_user_forbidden",
            })
        }
        Actor::User { .. } => {}
    }

    let trimmed = query_text.trim();
    if trimmed.is_empty() {
        return Err(WorkspaceSearchError::InvalidQuery { reason: "empty" });
    }
    if trimmed.chars().count() > MAX_QUERY_CHARS {
        return Err(WorkspaceSearchError::InvalidQuery { reason: "too_long" });
    }

    let limit = max_results.clamp(1, ABSOLUTE_MAX_RESULTS).max(1);
    let _ = DEFAULT_MAX_RESULTS; // documented default for callers; clamp handles bounds

    let mut q = crate::generated::UnifiedFieldSearchDocument::query(valence);
    q.inner = q
        .inner
        .set_search_fields(vec!["title".to_string(), "searchable_text".to_string()])
        .search(trimmed.to_string());

    let rows = q.await.map_err(|e| WorkspaceSearchError::Valence {
        operation: "query",
        message: e.to_string(),
    })?;

    Ok(rows
        .into_iter()
        .take(limit as usize)
        .map(|row| WorkspaceSearchHit {
            id: row.id().map(|r| r.id().to_string()).unwrap_or_default(),
            title: row.title().to_string(),
            description: None,
            app_id: row.app_id().to_string(),
            kind: row.kind().to_string(),
            link: row.link().to_string(),
        })
        .collect())
}
