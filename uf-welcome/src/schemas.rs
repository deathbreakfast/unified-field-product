//! Valence schema includes (SSR). Codegen inventory registers models at link time.

#[cfg(feature = "ssr")]
mod welcome_featured_app_schema {
    include!("../schemas/welcome_featured_app_valence_schema.rs");
}
