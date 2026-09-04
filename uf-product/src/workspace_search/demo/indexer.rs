//! SideEffect: keep [`crate::generated::UnifiedFieldSearchDocument`] in sync with IndexedDemoItem.

use async_trait::async_trait;
use valence::{Mutation, MutationKind, SideEffect};

use crate::generated::IndexedDemoItem;
use crate::workspace_search::{SearchDocumentDraft, SearchDocumentWriter};

/// App id written into index rows for the teaching demo source.
pub const DEMO_APP_ID: &str = "uf_product_demo";
/// Source table name (must match schema `table:`).
pub const DEMO_SOURCE_TABLE: &str = "indexed_demo_item";
/// Hit `kind` for AppBar grouping.
pub const DEMO_KIND: &str = "demo";

/// Title prefix that forces the indexer to return an error (K3 sad: SE must not roll back source).
pub const FORCE_SE_ERROR_TITLE_PREFIX: &str = "__force_se_error__";

/// Indexes [`IndexedDemoItem`] mutations into the workspace content index.
pub struct IndexedDemoIndexer;

#[async_trait]
impl SideEffect<IndexedDemoItem> for IndexedDemoIndexer {
    /// Keep the workspace search document in sync with the demo source row.
    ///
    /// # Errors
    ///
    /// Returns [`valence::Error::Internal`] when delete/mutate payloads are missing
    /// `before`/`after`, when the title starts with [`FORCE_SE_ERROR_TITLE_PREFIX`]
    /// (test harness), or when [`SearchDocumentWriter`] upsert/delete fails.
    async fn on_mutation(&self, mutation: &Mutation<'_, IndexedDemoItem>) -> valence::Result<()> {
        let v = mutation.valence();
        match *mutation.kind() {
            MutationKind::Delete => {
                let before = mutation.before().ok_or_else(|| {
                    valence::Error::Internal("IndexedDemoItem delete missing before".into())
                })?;
                let source_id = before.id().map(|r| r.id().to_string()).unwrap_or_default();
                SearchDocumentWriter::delete(
                    v,
                    before.user(),
                    DEMO_APP_ID,
                    DEMO_SOURCE_TABLE,
                    &source_id,
                )
                .await
                .map_err(|e| valence::Error::Internal(e.to_string()))?;
            }
            MutationKind::Create | MutationKind::Update => {
                let after = mutation.after().ok_or_else(|| {
                    valence::Error::Internal("IndexedDemoItem mutate missing after".into())
                })?;
                if after.title().starts_with(FORCE_SE_ERROR_TITLE_PREFIX) {
                    return Err(valence::Error::Internal(
                        "forced IndexedDemoIndexer failure for tests".into(),
                    ));
                }
                let source_id = after.id().map(|r| r.id().to_string()).unwrap_or_default();
                // Owner-scoped personal titles OK. Do NOT index emails, phones, secrets, tokens.
                SearchDocumentWriter::upsert(
                    v,
                    SearchDocumentDraft {
                        user: after.user().clone(),
                        app_id: DEMO_APP_ID.into(),
                        source_table: DEMO_SOURCE_TABLE.into(),
                        source_id,
                        title: after.title().to_string(),
                        searchable_text: after.title().to_string(),
                        link: after.link().to_string(),
                        kind: DEMO_KIND.into(),
                    },
                )
                .await
                .map_err(|e| valence::Error::Internal(e.to_string()))?;
            }
        }
        Ok(())
    }
}
