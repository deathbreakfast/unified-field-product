//! Product UI e2e host library.
#![allow(missing_docs)]

mod app;
#[cfg(feature = "ssr")]
mod app_state;
#[cfg(feature = "ssr")]
mod e2e_permissions;
#[cfg(feature = "ssr")]
mod e2e_spectra;
#[cfg(feature = "ssr")]
mod e2e_valence;
mod gate_demos;
mod harness_auth_menu;
mod help_steps;
mod pages;
#[cfg(feature = "ssr")]
mod photon_auth;
#[cfg(feature = "ssr")]
mod photon_boot;
#[cfg(feature = "ssr")]
pub mod seed;

pub use app::{shell, App};
#[cfg(feature = "ssr")]
pub use app_state::AppState;
#[cfg(feature = "ssr")]
pub use e2e_permissions::{
    wire_e2e_permissions, E2E_PERMISSION_ALLOW, E2E_PERMISSION_ALLOW_FLAG,
    E2E_PERMISSION_ALLOW_SESSION_KEY, E2E_PERMISSION_DENY,
};
#[cfg(feature = "ssr")]
pub use e2e_spectra::e2e_spectra;
#[cfg(feature = "ssr")]
pub use e2e_valence::{
    e2e_higgs_config, e2e_router, e2e_system_valence, e2e_valence, init_e2e_valence,
};
#[cfg(feature = "ssr")]
pub use gate_demos::inject_e2e_session_snapshot;
#[cfg(feature = "ssr")]
pub use photon_auth::E2ePhotonAuth;
#[cfg(feature = "ssr")]
pub use photon_boot::build_photon;
