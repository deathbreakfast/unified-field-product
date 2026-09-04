//! Client-safe search source DTOs (no SSR / Valence dependency).

use serde::{Deserialize, Serialize};

/// Identifies one registered search source (id + display label), as shown to the client
/// for source selection (e.g. checkboxes in a search-scope picker).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SearchSourceKey {
    /// Stable identifier, matched against `SearchSourceDescriptor::id` server-side (`ssr`).
    pub id: String,
    /// Human-readable label for display in source-selection UI.
    pub label: String,
}

impl SearchSourceKey {
    /// Construct a key from an id and display label.
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }
}

/// A single search result row, as returned to the client.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchSourceItem {
    /// Which registered source produced this result (matches [`SearchSourceKey::id`]).
    pub source_id: String,
    /// Result identifier, meaningful within `source_id` (e.g. a record id or route).
    pub id: String,
    /// Display title for the result row.
    pub title: String,
    /// Optional secondary text (e.g. a snippet or subtitle).
    pub description: Option<String>,
    /// Free-form result kind/category (e.g. `"app"`, `"notification"`), used for icon/grouping.
    pub kind: String,
}
