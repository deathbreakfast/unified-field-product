//! How close an indexed hit is to the searching user.

/// Closeness of a hit to the user, closest first. Stored as a small integer on the index row.
///
/// [`query`](super::query) orders hits by this value before the backend's text match.
/// Writers opt in through [`SearchDocumentWriter::upsert_with_relation`](super::SearchDocumentWriter::upsert_with_relation);
/// [`SearchDocumentWriter::upsert`](super::SearchDocumentWriter::upsert) writes [`SearchRelation::Other`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum SearchRelation {
    /// The user owns the source row.
    Owner,
    /// The source row is assigned to the user.
    Assignee,
    /// The user collaborates on the source row.
    Collaborator,
    /// No closer relation (default; also what rows written before this column read as).
    #[default]
    Other,
}

impl SearchRelation {
    /// Integer stored on the index row.
    #[must_use]
    pub fn as_i64(self) -> i64 {
        match self {
            Self::Owner => 0,
            Self::Assignee => 1,
            Self::Collaborator => 2,
            Self::Other => 3,
        }
    }

    /// Inverse of [`Self::as_i64`]; unknown values read as [`Self::Other`].
    #[must_use]
    pub fn from_i64(value: i64) -> Self {
        match value {
            0 => Self::Owner,
            1 => Self::Assignee,
            2 => Self::Collaborator,
            _ => Self::Other,
        }
    }
}
