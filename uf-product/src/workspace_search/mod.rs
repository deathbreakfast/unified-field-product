//! Per-user workspace **content index** (AppBar search) — not picker search sources.
//!
//! Stores index rows, provides a writer and owner-scoped query on SSR, and
//! exposes [`query_workspace_search`] for the AppBar UI in `uf-integrations`.
//! Picker registry contracts live in [`crate::search_sources`] / `uf-search-core`.
//!
//! The teaching stub [`demo`] demonstrates IndexedDemoItem SE/Iter patterns;
//! production app source models register their own SideEffect / iters entries.
//!
//! ## Integrating a source model
//!
//! Apps register a `SideEffect` / `iters` entry on the **source** schema and call
//! [`SearchDocumentWriter`] (see [`demo::IndexedDemoIndexer`] / [`demo::IndexedDemoBackfillIter`]).
//!
//! ## Privacy
//!
//! Index rows use `OWNER_BY_USER_FIELD` read and `SYSTEM_ONLY` CUD. [`query`] rejects
//! Anonymous and System actors so a mistaken `system_valence()` call cannot dump every
//! user's documents.
//!
//! ## Indexing policy
//!
//! Owner-scoped resource titles are allowed. Do **not** put emails, phone numbers,
//! secrets, session tokens, or auth material in `title` / `searchable_text` / `link`.
//!
//! ## Concern → API
//!
//! | Concern | API |
//! |---------|-----|
//! | Upsert / delete index rows | [`SearchDocumentWriter`], [`SearchDocumentDraft`] |
//! | Query my hits (library) | [`query`], [`WorkspaceSearchHit`] |
//! | Query from UI | [`query_workspace_search`] |
//! | Errors | [`WorkspaceSearchError`] |
//! | TTL window | [`SEARCH_DOCUMENT_TTL_SECS`] |
//! | Teaching SE / Iter | [`demo`] |
//!
//! # Example
//!
//! Build a [`SearchDocumentDraft`] and upsert through [`SearchDocumentWriter`]
//! (see [`demo::IndexedDemoIndexer`] for SideEffect / Iter wiring), then query:
//!
//! ```rust,ignore
//! use uf_product::workspace_search::{query, SearchDocumentWriter, WorkspaceSearchHit};
//! // let draft: SearchDocumentDraft = /* user, app_id, source_table, title, link, … */;
//! // SearchDocumentWriter::upsert(&valence, draft).await?;
//! let hits: Vec<WorkspaceSearchHit> = query(&valence, "title", 10).await?;
//! ```
//!
//! # Errors
//!
//! [`WorkspaceSearchError`] covers unauthenticated callers, invalid actors,
//! rejected query/link text, and Valence persistence failures. The AppBar server
//! fn [`query_workspace_search`] maps these to [`leptos::prelude::ServerFnError`].

mod dto;
#[cfg(any(feature = "ssr", feature = "hydrate"))]
mod server;

#[cfg(feature = "ssr")]
pub mod demo;
#[cfg(feature = "ssr")]
mod draft;
#[cfg(feature = "ssr")]
mod error;
#[cfg(feature = "ssr")]
mod link;
#[cfg(feature = "ssr")]
mod query;
#[cfg(feature = "ssr")]
mod writer;

pub use dto::WorkspaceSearchHit;
#[cfg(any(feature = "ssr", feature = "hydrate"))]
pub use server::query_workspace_search;

#[cfg(feature = "ssr")]
pub use draft::SearchDocumentDraft;
#[cfg(feature = "ssr")]
pub use error::WorkspaceSearchError;
#[cfg(feature = "ssr")]
pub use query::query;
#[cfg(feature = "ssr")]
pub use writer::{document_id, SearchDocumentWriter, SEARCH_DOCUMENT_TTL_SECS};

#[cfg(all(test, feature = "ssr"))]
mod tests;
