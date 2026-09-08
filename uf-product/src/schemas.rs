//! Product Valence trait / overlay registration.
//!
//! Entity schemas (`session`, `user_appearance`, `help_tour_step_visit`,
//! `unified_field_search_document`, `IndexedDemoItem`) are registered only via
//! build.rs codegen (`generated_models.rs`). Do not `include!` those
//! `valence_schema!` sources here (Gauge pattern: avoid dual macro+codegen
//! registration that panics in [`valence::SchemaRegistry`]).

// No product-local trait overlays yet.
