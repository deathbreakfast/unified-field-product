//! Leptos-free types for the `/search` command palette: source keys and items shared with the
//! client, plus (behind `ssr`) the backend provider trait and inventory-based registry.
//!
//! Client DTOs ([`SearchSourceKey`], [`SearchSourceItem`]) have no SSR-only
//! dependencies. The Leptos combobox UI lives in `uf-integrations::SearchSourcePicker`.
//! Macro expansion for `define_search_sources!` is in `uf-product-macros`; app crates
//! typically import through `uf-product::search_sources`.
//!
//! ## Features
//!
//! - **Backend search registry** — `SearchSourceProvider` and `inventory::submit!` register
//!   one backend source per descriptor so SSR hosts discover providers without a central enum.
//!   [Get started](#getting-started)
//! - **Multi-source fan-out** — [`SearchSourceRegistry`] fans a query across selected
//!   [`SearchSourceKey`] values and returns merged [`SearchSourceItem`] rows for pickers.
//!   [Get started](#query-many-sources)
//! - **Client-safe DTOs** — [`SearchSourceKey`] and [`SearchSourceItem`] serialize to the
//!   browser with no Valence or inventory dependency.
//!
//! ## Getting started
//!
//! The backend search registry collects one [`SearchSourceProvider`] per source type at
//! process start. Each provider registers through `inventory::submit!` so the SSR host
//! discovers sources without maintaining a hand-written list.
//!
//! **Prerequisites:** Enable the `ssr` feature on this crate and depend on `valence` in the
//! host that runs search queries.
//!
//! ```rust,ignore
//! use uf_search_core::{SearchSourceDescriptor, SearchSourceFuture, SearchSourceItem, SearchSourceProvider};
//!
//! struct MyProvider;
//! impl SearchSourceProvider for MyProvider {
//!     fn query<'a>(
//!         &'a self,
//!         valence: &'a valence::Valence,
//!         query_text: &'a str,
//!         max_results: u32,
//!     ) -> SearchSourceFuture<'a> {
//!         Box::pin(async move { Ok(Vec::<SearchSourceItem>::new()) })
//!     }
//! }
//!
//! static PROVIDER: MyProvider = MyProvider;
//! quark::inventory::submit! {
//!     SearchSourceDescriptor {
//!         id: "my-source",
//!         label: "My Source",
//!         description: "Searches my stuff",
//!         provider: &PROVIDER,
//!     }
//! }
//! ```
//!
//! On success the static [`SearchSourceDescriptor`] is collected at startup and
//! [`SearchSourceProvider::query`] returns `Ok(rows)` with [`SearchSourceItem`] hits.
//! Use `uf-product-macros::define_search_sources!` when you prefer macro-generated
//! descriptors over hand-written `submit!` blocks.
//!
//! ## Query many sources
//!
//! Multi-source fan-out runs one query across selected [`SearchSourceKey`] values through
//! [`SearchSourceRegistry::query_many`]. Call it from server functions that back search
//! pickers when a user types a query string.
//!
//! **Prerequisites:** At least one [`SearchSourceDescriptor`] registered (see
//! [Getting started](#getting-started)); a [`valence::Valence`] handle for provider queries.
//!
//! ```rust,ignore
//! use uf_search_core::{SearchSourceItem, SearchSourceKey, SearchSourceRegistry};
//!
//! async fn search(
//!     registry: &SearchSourceRegistry,
//!     valence: &valence::Valence,
//! ) -> Result<Vec<SearchSourceItem>, uf_search_core::SearchSourceError> {
//!     let keys = vec![SearchSourceKey::new("apps", "Apps")];
//!     let hits = registry.query_many(&keys, valence, "counter", 10).await?;
//!     assert!(!hits.is_empty());
//!     assert!(hits[0].title.len() > 0);
//!     Ok(hits)
//! }
//! ```
//!
//! On success `query_many` returns merged [`SearchSourceItem`] rows capped at `max_results`.
//! Unknown source ids are skipped; the first provider error stops fan-out with
//! [`SearchSourceError::source_id`] set. See the `query_many_stub` example for a runnable
//! stub provider.
//!
//! ## Feature flags
//!
//! | Feature | Effect |
//! |---------|--------|
//! | `default` | Client DTOs only ([`SearchSourceKey`], [`SearchSourceItem`]). |
//! | `ssr` | Enables [`SearchSourceProvider`], [`SearchSourceDescriptor`], [`SearchSourceRegistry`], and Valence-backed fan-out. |
//!
//! ## Examples
//!
//! Start with `SearchSourceProvider` + `inventory::submit!` in
//! [Getting started](#getting-started). Fan-out `SearchSourceRegistry::query_many` in
//! workspace `uf-search-core/examples/query_many_stub.rs`. Combobox over these DTOs:
//! `uf-integrations::SearchSourcePicker`.
//!
//! ```bash
//! cargo run -p uf-search-core --example query_many_stub --features ssr
//! ```
//!
//! ## Where to look next
//!
//! - [`SearchSourceKey`] / [`SearchSourceItem`] — client-visible DTOs.
//! - [`SearchSourceRegistry`] (`ssr`) — the fan-out query registry.
//! - `uf-product-macros` — `define_search_sources!` to emit descriptors.
//! - `uf-integrations` — `SearchSourcePicker` UI over these DTOs.
//! - `uf-product::search_sources` — product re-export for app crates.

mod dto;
#[cfg(feature = "ssr")]
mod ssr;

pub use dto::{SearchSourceItem, SearchSourceKey};

#[cfg(feature = "ssr")]
pub use ssr::{
    SearchSourceDescriptor, SearchSourceError, SearchSourceFuture, SearchSourceProvider,
    SearchSourceRegistry, SearchSourceResult,
};
