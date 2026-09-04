//! Draft payload for writing a workspace search document.

use valence::RecordId;

/// Input to [`super::SearchDocumentWriter::upsert`].
///
/// # Indexing policy
///
/// - Set [`Self::user`] to the owning user record.
/// - Prefer non-secret resource titles in [`Self::title`] / [`Self::searchable_text`].
/// - Do **not** index emails, phone numbers, secrets, or auth tokens.
/// - [`Self::link`] must be an app-relative path (`/…`).
#[derive(Debug, Clone)]
pub struct SearchDocumentDraft {
    /// Owning user (`Record("user")`).
    pub user: RecordId,
    /// Product / app id that owns the source row.
    pub app_id: String,
    /// Source Valence table name.
    pub source_table: String,
    /// Source row id (within `source_table`).
    pub source_id: String,
    /// Display title in AppBar results.
    pub title: String,
    /// Text searched by [`super::query`] (title and/or safe excerpt).
    pub searchable_text: String,
    /// In-app navigation path (relative).
    pub link: String,
    /// Result kind for UI grouping (e.g. `"todo"`, `"demo"`).
    pub kind: String,
}
