//! Iter: backfill missing / stale workspace search documents for IndexedDemoItem.

use valence::{IterEvaluation, Model, Valence};

use crate::generated::{IndexedDemoItem, UnifiedFieldSearchDocument};
use crate::workspace_search::demo::indexer::{DEMO_APP_ID, DEMO_KIND, DEMO_SOURCE_TABLE};
use crate::workspace_search::{document_id, SearchDocumentDraft, SearchDocumentWriter};

/// Backfills [`UnifiedFieldSearchDocument`] rows for [`IndexedDemoItem`] sources.
pub struct IndexedDemoBackfillIter;

impl IndexedDemoBackfillIter {
    /// Run when the index row is missing or title/link drifted from the source.
    ///
    /// # Errors
    ///
    /// Returns [`valence::Error`] when the System-elevated index `get` fails.
    /// A source row without an id skips (`Ok(IterEvaluation::skip)`), not `Err`.
    pub async fn should_run(
        &self,
        row: &IndexedDemoItem,
        valence: &Valence,
    ) -> valence::Result<IterEvaluation> {
        let Some(rid) = row.id() else {
            return Ok(IterEvaluation::skip("source missing id"));
        };
        let id = document_id(row.user(), DEMO_APP_ID, DEMO_SOURCE_TABLE, rid.id());
        // Prefer job/System Valence already in hand; only rebind when the caller is not System.
        let index_v = if valence.actor().is_system() {
            valence.clone()
        } else {
            valence.with_actor(valence::Actor::System {
                operation: "indexed_demo_backfill_should_run".into(),
            })
        };
        match UnifiedFieldSearchDocument::get(&id, &index_v).await? {
            Some(doc)
                if doc.title() == row.title()
                    && doc.link() == row.link()
                    && doc.app_id() == DEMO_APP_ID =>
            {
                Ok(IterEvaluation::skip("index fresh"))
            }
            _ => Ok(IterEvaluation::run(
                "index missing or stale within retention window",
            )),
        }
    }

    /// Upsert the index row for this source (same mapping as the SideEffect).
    ///
    /// # Errors
    ///
    /// Returns [`valence::Error::Internal`] when the source row has no id, or when
    /// [`SearchDocumentWriter::upsert`] fails (mapped into `Internal`).
    pub async fn execute(&self, row: &IndexedDemoItem, valence: &Valence) -> valence::Result<()> {
        let Some(rid) = row.id() else {
            return Err(valence::Error::Internal(
                "IndexedDemoItem backfill missing id".into(),
            ));
        };
        SearchDocumentWriter::upsert(
            valence,
            SearchDocumentDraft {
                user: row.user().clone(),
                app_id: DEMO_APP_ID.into(),
                source_table: DEMO_SOURCE_TABLE.into(),
                source_id: rid.id().to_string(),
                title: row.title().to_string(),
                searchable_text: row.title().to_string(),
                link: row.link().to_string(),
                kind: DEMO_KIND.into(),
            },
        )
        .await
        .map_err(|e| valence::Error::Internal(e.to_string()))
    }
}
