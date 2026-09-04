//! Appearance server fns, context bootstrap, and permission-toast helpers.
//!
//! Session loading for auth lives in [`crate::session`] (`get_session` /
//! `init_auth_resource`); host credential stores stay in lepton-auth.
//! Preference types and localStorage helpers live in [`crate::theme`].
//! Shell appearance menu UI is composed in `uf-integrations`.
//!
//! # Concern → API
//!
//! | Concern | API |
//! |---------|-----|
//! | Load / persist signed-in appearance | [`get_my_appearance`], [`save_my_appearance`] |
//! | Provide prefs near the app root | [`provide_appearance_context`], [`init_appearance_resource`] |
//! | Re-export theme hooks for one import path | [`use_appearance_preferences`], [`AppearanceContext`] |
//! | Permission toast requests from shell layout | [`permission_server_errors`] ([`PermissionServerError`], [`report_server_fn_error`]) |
//!
//! # Example
//!
//! ```rust,ignore
//! use leptos::prelude::*;
//! use uf_product::services::{
//!     init_appearance_resource, provide_appearance_context, save_my_appearance,
//! };
//!
//! // Near the host root (hydrate / ssr):
//! let appearance_ctx = provide_appearance_context();
//! let _appearance = init_appearance_resource(appearance_ctx);
//!
//! // After the user picks dark mode:
//! let _ = save_my_appearance("dark".into(), "product".into(), None);
//! ```
//!
//! # Errors
//!
//! [`get_my_appearance`] and [`save_my_appearance`] return [`leptos::prelude::ServerFnError`]
//! when the caller is not authenticated, Higgs/Valence setup fails, or persistence
//! rejects the payload (for example a malformed brand seed color).

pub mod appearance_service;
pub mod permission_server_errors;

pub use crate::theme::{
    use_appearance_preferences, AppearanceContext, AppearancePreferences, APPEARANCE_STORAGE_KEY,
    PRODUCT_BRAND_PRESETS,
};
pub use appearance_service::{
    get_my_appearance, init_appearance_resource, provide_appearance_context, save_my_appearance,
    AppearanceData,
};
pub use permission_server_errors::{
    parse_permission_server_error, provide_permission_toast_bus, report_server_fn_error,
    report_server_fn_error_with_bus, use_permission_toast_bus, PermissionServerError,
    PermissionToastBus, PermissionToastRequest,
};
