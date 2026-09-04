// Product Valence schema registration (auth/user tables live in lepton-identity).

#[cfg(feature = "ssr")]
mod session_schema {
    include!("../schemas/session_valence_schema.rs");
}

#[cfg(feature = "ssr")]
mod user_appearance_schema {
    include!("../schemas/user_appearance_valence_schema.rs");
}

#[cfg(feature = "ssr")]
mod help_tour_step_visit_schema {
    include!("../schemas/help_tour_step_visit_valence_schema.rs");
}

#[cfg(feature = "ssr")]
mod unified_field_search_document_schema {
    include!("../schemas/unified_field_search_document_valence_schema.rs");
}

// IndexedDemoItem is registered only via build.rs codegen (`generated_models.rs`) so
// `side_effects` / `iters` inventory hooks resolve against `crate::generated` imports.
// Do not `include!` the schema here (Gauge pattern: avoid dual macro+codegen registration).
