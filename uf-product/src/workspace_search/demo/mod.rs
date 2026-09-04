//! Teaching source model SideEffect + Iter for [`super::SearchDocumentWriter`].

mod backfill_iter;
mod indexer;

pub use backfill_iter::IndexedDemoBackfillIter;
pub use indexer::{IndexedDemoIndexer, DEMO_APP_ID, DEMO_KIND, DEMO_SOURCE_TABLE};

#[cfg(all(test, feature = "ssr"))]
mod tests;
