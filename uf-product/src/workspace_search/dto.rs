//! Client-safe workspace search hit (no owning user id).

use serde::{Deserialize, Serialize};

/// One AppBar / Dialog result row.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceSearchHit {
    /// Index document id.
    pub id: String,
    /// Display title.
    pub title: String,
    /// Optional secondary text (often mirrors a short description).
    pub description: Option<String>,
    /// App that contributed the hit.
    pub app_id: String,
    /// Result kind for UI.
    pub kind: String,
    /// App-relative navigation path.
    pub link: String,
}
