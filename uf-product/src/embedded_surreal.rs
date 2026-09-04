//! Logical database name/storage constants for uf-product Valence schemas.
//!
//! Engine id is selected by Cargo feature: `db-sqlite` (default) or `db-hybrid`.

use valence::{Database, DatabaseFromEngine};

/// Logical name for identity-related product schemas.
pub const IDENTITY_LOGICAL_NAME: &str = "default";
/// Logical name for session-related product schemas.
pub const SESSION_LOGICAL_NAME: &str = "default";

#[cfg(feature = "db-hybrid")]
const ENGINE_ID: &str = valence::HYBRID_ENGINE_ID;

#[cfg(not(feature = "db-hybrid"))]
const ENGINE_ID: &str = valence::SQLITE_ENGINE_ID;

/// Identity schemas storage (`"{engine}:default"`).
pub const IDENTITY_DEFAULT_STORAGE: DatabaseFromEngine =
    Database::from_engine(IDENTITY_LOGICAL_NAME, ENGINE_ID);

/// Session schemas storage (`"{engine}:default"`).
pub const SESSION_DEFAULT_STORAGE: DatabaseFromEngine =
    Database::from_engine(SESSION_LOGICAL_NAME, ENGINE_ID);

/// Logical names hosts should register for these schemas.
pub const EMBEDDED_SURREAL_LOGICAL_NAMES: &[&str] = &[IDENTITY_LOGICAL_NAME];
