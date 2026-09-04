//! System-actor writer for [`crate::generated::UnifiedFieldSearchDocument`].

use chrono::Utc;
use valence::{Actor, Model, RecordId, Valence};

use super::link::validate_relative_link;
use super::{SearchDocumentDraft, WorkspaceSearchError};

/// Schema TTL window (90 days), matching `ttl.seconds` on the Valence schema.
pub const SEARCH_DOCUMENT_TTL_SECS: u64 = 7_776_000;

const INDEX_TABLE: &str = "unified_field_search_document";

/// Upsert / delete helpers. Always elevates to [`Actor::System`] for CUD.
pub struct SearchDocumentWriter;

impl SearchDocumentWriter {
    /// Deterministic document id for `(user, app_id, source_table, source_id)`.
    #[must_use]
    pub fn document_id(
        user: &RecordId,
        app_id: &str,
        source_table: &str,
        source_id: &str,
    ) -> String {
        document_id(user, app_id, source_table, source_id)
    }

    /// Create or replace an index row. Physically removes any existing row first so Valence
    /// TTL is restamped on the create path (schema TTL is create-only; this is sliding retention).
    ///
    /// Uses a hard backend delete rather than [`Model::delete`], which only marks
    /// `pending_deletion` and would block recreate until a host deletion worker runs.
    ///
    /// # Errors
    ///
    /// Returns [`WorkspaceSearchError::InvalidLink`] for unsafe links, or
    /// [`WorkspaceSearchError::Write`] when Valence CUD fails.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use uf_product::workspace_search::{SearchDocumentDraft, SearchDocumentWriter};
    /// use valence::RecordId;
    ///
    /// # async fn demo(v: &valence::Valence) -> Result<(), uf_product::workspace_search::WorkspaceSearchError> {
    /// SearchDocumentWriter::upsert(
    ///     v,
    ///     SearchDocumentDraft {
    ///         user: RecordId::new("user", "alice"),
    ///         app_id: "demo".into(),
    ///         source_table: "indexed_demo_item".into(),
    ///         source_id: "1".into(),
    ///         title: "Beacon Checklist".into(),
    ///         searchable_text: "Beacon Checklist".into(),
    ///         link: "/demo/1".into(),
    ///         kind: "demo".into(),
    ///     },
    /// )
    /// .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn upsert(
        valence: &Valence,
        draft: SearchDocumentDraft,
    ) -> Result<(), WorkspaceSearchError> {
        validate_relative_link(&draft.link)?;
        if draft.user.table() != "user" {
            return Err(WorkspaceSearchError::Write {
                operation: "upsert",
                source_table: draft.source_table.clone(),
                message: "user record must use table `user`".into(),
            });
        }
        if draft.title.trim().is_empty() || draft.searchable_text.trim().is_empty() {
            return Err(WorkspaceSearchError::Write {
                operation: "upsert",
                source_table: draft.source_table.clone(),
                message: "title and searchable_text are required".into(),
            });
        }

        let id = document_id(
            &draft.user,
            &draft.app_id,
            &draft.source_table,
            &draft.source_id,
        );
        let system = system_valence(valence, "workspace_search_upsert");
        let source_table = draft.source_table.clone();

        hard_remove_document(&system, &id, "upsert", &source_table).await?;

        let row = crate::generated::UnifiedFieldSearchDocument::new(
            draft.user,
            draft.app_id,
            draft.source_table,
            draft.source_id,
            draft.title,
            draft.searchable_text,
            draft.link,
            draft.kind,
            Utc::now(),
        )
        .map_err(|e| WorkspaceSearchError::Write {
            operation: "upsert",
            source_table: source_table.clone(),
            message: e.to_string(),
        })?;

        crate::generated::UnifiedFieldSearchDocument::upsert(&id, row, &system)
            .await
            .map_err(|e| WorkspaceSearchError::Write {
                operation: "upsert",
                source_table,
                message: e.to_string(),
            })?;
        Ok(())
    }

    /// Delete an index row by natural key. Missing rows are a no-op success.
    ///
    /// # Errors
    ///
    /// Returns [`WorkspaceSearchError::Write`] when delete fails for a reason other
    /// than not found.
    pub async fn delete(
        valence: &Valence,
        user: &RecordId,
        app_id: &str,
        source_table: &str,
        source_id: &str,
    ) -> Result<(), WorkspaceSearchError> {
        let id = document_id(user, app_id, source_table, source_id);
        let system = system_valence(valence, "workspace_search_delete");
        hard_remove_document(&system, &id, "delete", source_table).await
    }
}

/// Deterministic document id for the natural key.
#[must_use]
pub fn document_id(user: &RecordId, app_id: &str, source_table: &str, source_id: &str) -> String {
    // Keep ids filesystem/backend-safe and unique per owner + source.
    format!(
        "{}__{}__{}__{}",
        sanitize_id_part(user.id()),
        sanitize_id_part(app_id),
        sanitize_id_part(source_table),
        sanitize_id_part(source_id)
    )
}

fn sanitize_id_part(part: &str) -> String {
    part.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => c,
            _ => '_',
        })
        .collect()
}

fn system_valence(valence: &Valence, operation: &str) -> Valence {
    valence.with_actor(Actor::System {
        operation: operation.to_string(),
    })
}

async fn hard_remove_document(
    system: &Valence,
    id: &str,
    operation: &'static str,
    source_table: &str,
) -> Result<(), WorkspaceSearchError> {
    let backend =
        system
            .backend_for_table(INDEX_TABLE)
            .map_err(|e| WorkspaceSearchError::Write {
                operation,
                source_table: source_table.to_string(),
                message: e.to_string(),
            })?;
    match backend.delete_record(INDEX_TABLE, id).await {
        Ok(()) => {}
        Err(valence::Error::NotFound(_)) => {}
        Err(e) => {
            return Err(WorkspaceSearchError::Write {
                operation,
                source_table: source_table.to_string(),
                message: e.to_string(),
            });
        }
    }
    valence::read_cache::invalidate(INDEX_TABLE, id);
    let _ =
        valence::ownership::OwnershipService::mark_deleted_ownership(INDEX_TABLE, id, system).await;
    Ok(())
}
