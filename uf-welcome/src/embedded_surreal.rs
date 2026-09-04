//! Logical embedded database name for uf-welcome Valence schemas.

use valence::{Database, DatabaseFromEngine, MEM_ENGINE_ID};

/// Logical database name welcome schemas are registered under.
pub const DEFAULT_LOGICAL_NAME: &str = "default";

/// [`DatabaseFromEngine`] pointing at [`DEFAULT_LOGICAL_NAME`] on the embedded mem engine.
///
/// Featured catalog CRUD uses ownership-gated Model APIs; the mem backend implements
/// the ownership bundle query that SQLite does not.
pub const DEFAULT_STORAGE: DatabaseFromEngine =
    Database::from_engine(DEFAULT_LOGICAL_NAME, MEM_ENGINE_ID);

/// Logical names test/server routers should link for `uf-welcome` models to resolve.
pub const EMBEDDED_SURREAL_LOGICAL_NAMES: &[&str] = &[DEFAULT_LOGICAL_NAME];
