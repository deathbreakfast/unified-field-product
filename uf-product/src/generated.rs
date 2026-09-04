#![allow(
    dead_code,
    unused_imports,
    missing_docs,
    non_snake_case,
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::restriction
)]
//! Valence models generated from `schemas/*.json` by `build.rs` (via `valence_codegen`).
//!
//! Contents are produced at build time and intentionally left undocumented here; see the
//! schema JSON files for field-level semantics.

use valence::privacy_policies::common::{AUTHENTICATED, PUBLIC_READ, SYSTEM_ONLY};
use valence::privacy_policies::owner::{OWNER_BY_ID, OWNER_BY_USER_FIELD};

use crate::workspace_search::demo::{IndexedDemoBackfillIter, IndexedDemoIndexer};

include!(concat!(env!("OUT_DIR"), "/generated_models.rs"));
