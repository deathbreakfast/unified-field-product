//! SSR app registration metadata and inventory registry.
//!
//! Product apps usually register through `uf_app!`, which submits an
//! [`AppRegistration`] into [`AppRegistry`]. The host still mounts the route tree;
//! this module only carries discovery metadata (name, slug, route prefix, optional
//! docs links, optional permission manifest hook).
//!
//! # Example
//!
//! ```rust,ignore
//! use uf_product::routes::{AppRegistry, get_all_app_route_paths};
//!
//! let apps = AppRegistry::auto_discover();
//! assert!(!apps.is_empty());
//! let paths = get_all_app_route_paths();
//! assert!(paths.iter().any(|p| *p == "/counter"));
//! ```
//!
//! Runnable inventory proof: `cargo run -p uf-product --example uf_app_registration --features ssr`.

/// Registration entry for a single Orbital application.
///
/// App crates usually create this indirectly via `uf_app!`. The registration is
/// collected through inventory on SSR builds so the shell can:
///
/// - list available apps,
/// - build application switchers and navigation,
/// - discover startup permission manifests, and
/// - associate route prefixes with human-friendly app metadata.
///
/// The route component tree itself is still composed by the root `app` crate; this
/// type only captures the metadata needed for discovery and shell integration.
pub struct AppRegistration {
    /// Unique application identifier, such as `"counter"` or `"permission"`.
    pub id: &'static str,

    /// Human-friendly application name shown in Orbital UI.
    pub name: &'static str,

    /// Short app description used in discovery surfaces.
    pub description: &'static str,

    /// App icon identifier understood by Orbital UI surfaces.
    pub icon: &'static str,

    /// Primary route prefix for the app, such as `"/counter"`.
    pub route_path: &'static str,

    /// Optional source repository URL (for example a GitHub repo page).
    pub repository: Option<&'static str>,

    /// Optional crates.io package name. When set, docs link to `https://docs.rs/{crate_name}`.
    pub crate_name: Option<&'static str>,

    /// Optional per-app brand seed override (`#RRGGBB`).
    pub brand_seed: Option<&'static str>,

    /// Optional app-declared permission manifest used for startup synchronization.
    pub permission_manifest: Option<fn() -> &'static crate::permissions::AppPermissionManifest>,
}

crate::inventory::collect!(AppRegistration);

impl quark::Registrable for AppRegistration {
    fn registry_key(&self) -> &str {
        self.id
    }
}

quark::define_registry! {
    /// Registry of all apps discovered via `uf_app!`.
    ///
    /// The root app shell can call `auto_discover()` to enumerate every registered
    /// Orbital application without keeping a hard-coded list in one place.
    pub struct AppRegistry for AppRegistration;
}

/// Return the route prefixes for all registered Orbital apps.
///
/// This is useful when building shell navigation, validating route ownership, or
/// deriving allow-lists for app-aware features.
pub fn get_all_app_route_paths() -> Vec<&'static str> {
    AppRegistry::auto_discover()
        .iter()
        .map(|registration| registration.route_path)
        .collect()
}
