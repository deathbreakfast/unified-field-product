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
//! Valence-codegen output for uf-welcome schemas (`build.rs` + `schemas/`).
//! Generated model types are not hand-documented; see `../schemas/*.rs` for the
//! source-of-truth field definitions.

#[cfg(feature = "ssr")]
use valence::privacy_policies::common::{AUTHENTICATED, SYSTEM_ONLY};

#[cfg(feature = "ssr")]
include!(concat!(env!("OUT_DIR"), "/generated_models.rs"));
