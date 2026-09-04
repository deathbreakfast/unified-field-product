//! Search source contracts used by product pickers and global search UIs.
//!
//! Search sources let an app expose backend-backed search results in a consistent
//! shape so UI components can query many resource types through one picker.
//! Typical examples include user lookup, permission-group lookup, Gluon image search,
//! or node selection during deployment workflows.
//!
//! This module re-exports the core search types from [`uf_search_core`].
//!
//! ## How it fits together
//!
//! 1. An app declares one or more source ids, usually with `define_search_sources!`.
//! 2. Each source implements [`SearchSourceProvider`] to query Valence-backed data
//!    (`?` maps Valence failures into [`SearchSourceError`]).
//! 3. UI components submit the selected [`SearchSourceKey`] values and render the
//!    returned [`SearchSourceItem`] list. Registry fan-out is
//!    [`SearchSourceRegistry::query_many`].
//!
//! ## Example
//!
//! ```rust,ignore
//! uf_product_macros::define_search_sources! {
//!     enum PermissionSearchSourceId {
//!         User => {
//!             id: "user_search_source",
//!             label: "Users",
//!             description: "Searches user principals",
//!             provider: PlatformUserSearchSource
//!         }
//!     }
//! }
//!
//! #[cfg(feature = "ssr")]
//! pub struct PlatformUserSearchSource;
//!
//! #[cfg(feature = "ssr")]
//! impl uf_product::search_sources::SearchSourceProvider for PlatformUserSearchSource {
//!     fn query<'a>(
//!         &'a self,
//!         valence: &'a valence::Valence,
//!         query_text: &'a str,
//!         max_results: u32,
//!     ) -> uf_product::search_sources::SearchSourceFuture<'a> {
//!         Box::pin(async move {
//!             let users = lepton::generated::User::query(valence)
//!                 .where_email(valence::StringPredicate::Contains(query_text.to_string()))
//!                 .limit(max_results)
//!                 .await?;
//!
//!             Ok(users
//!                 .into_iter()
//!                 .map(|user| uf_product::search_sources::SearchSourceItem {
//!                     source_id: "user_search_source".to_string(),
//!                     id: user.id().unwrap_or_default().to_string(),
//!                     title: user.email().to_string(),
//!                     description: Some("User account".to_string()),
//!                     kind: "user".to_string(),
//!                 })
//!                 .collect())
//!         })
//!     }
//! }
//! ```
//!
//! UI picker: `uf_integrations::SearchSourcePicker`. Registration macros live in
//! `uf_product_macros`. Runtime contracts and registry: `uf_search_core`.

pub use uf_search_core::*;
