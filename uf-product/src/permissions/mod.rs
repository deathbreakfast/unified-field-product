//! Permission manifests and contracts used by Orbital applications.
//!
//! Apps declare a stable permission surface so UI routes, server functions, and
//! management tooling share the same names. Manifest shapes are defined here;
//! Gauge evaluation and host credential stores live elsewhere. Runtime
//! allow/deny is fail-closed in [`crate::routes`] until Gauge is wired.
//!
//! # Concern → API
//!
//! | Concern | API |
//! |---------|-----|
//! | App-level permission catalog | [`AppPermissionManifest`] |
//! | Group permissions into domains | [`PermissionDomainSpec`], [`PermissionSpec`] |
//! | Enum → canonical string names | [`PermissionEnum`] |
//! | Macro / registry lookup of an app's manifest | [`AppPermissionManifestProvider`] |
//!
//! # Example
//!
//! ```rust
//! use uf_product::{
//!     AppPermissionManifest, AppPermissionManifestProvider, PermissionDomainSpec, PermissionEnum,
//!     PermissionSpec,
//! };
//!
//! #[derive(Clone, Copy)]
//! enum CounterPermission {
//!     ViewAdmin,
//!     SetCounter,
//! }
//!
//! impl PermissionEnum for CounterPermission {
//!     fn as_str(self) -> &'static str {
//!         match self {
//!             Self::ViewAdmin => "counter.admin.view",
//!             Self::SetCounter => "counter.admin.set",
//!         }
//!     }
//!
//!     fn all() -> &'static [Self] {
//!         &[Self::ViewAdmin, Self::SetCounter]
//!     }
//! }
//!
//! static COUNTER_PERMISSIONS: &[PermissionSpec] = &[
//!     PermissionSpec {
//!         name: "counter.admin.view",
//!         description: "View the counter administration screen",
//!     },
//!     PermissionSpec {
//!         name: "counter.admin.set",
//!         description: "Change the global counter value",
//!     },
//! ];
//!
//! static COUNTER_DOMAINS: &[PermissionDomainSpec] = &[PermissionDomainSpec {
//!     key: "counter_admin",
//!     name: "Counter Admin",
//!     description: "Administrative actions for the counter app",
//!     permissions: COUNTER_PERMISSIONS,
//! }];
//!
//! static COUNTER_MANIFEST: AppPermissionManifest = AppPermissionManifest {
//!     app_id: "counter",
//!     domains: COUNTER_DOMAINS,
//! };
//!
//! struct CounterPermissionManifest;
//!
//! impl AppPermissionManifestProvider for CounterPermissionManifest {
//!     fn manifest() -> &'static AppPermissionManifest {
//!         &COUNTER_MANIFEST
//!     }
//! }
//!
//! assert_eq!(CounterPermissionManifest::manifest().app_id, "counter");
//! ```
//!
//! In most Orbital apps this manifest is generated from `#[derive(UfPermissionManifest)]`
//! or referenced from `uf_app!`, but the runtime shape is defined here.
//!
//! # Runtime checks
//!
//! Hosts install a `PermissionBackend` via `provide_permission_backend` at
//! shell bootstrap (SSR). `require_permission` and route gates **fail closed** when
//! no backend is in context.
//!
//! Sensitive mutations that need a recent TOTP check use `require_step_up`
//! (or `step_up` on `#[uf_product_macros::server]`). lepton-auth opens the
//! session sudo window; this module only reads it.

mod backend;
#[cfg(feature = "ssr")]
mod step_up;

#[cfg(feature = "ssr")]
pub use backend::{
    eval_permission_by_name, has_permission, provide_permission_backend, require_permission,
    use_permission_backend, PermissionBackend,
};

pub use backend::check_permission_by_name;

#[cfg(feature = "ssr")]
pub use step_up::{
    provide_step_up_backend, require_step_up, use_step_up_backend, StepUpBackend, StepUpMode,
    STEP_UP_AUTH_HASH_KEY, STEP_UP_EXPIRES_AT_KEY, STEP_UP_SCOPE_KEY, STEP_UP_SCOPE_SENSITIVE,
    STEP_UP_TTL_SECS, STEP_UP_USER_ID_KEY, STEP_UP_VERIFIED_AT_KEY,
};

/// Convert app-specific permission enums into canonical string names.
///
/// This trait is intentionally small: Orbital only needs a stable identifier for each permission plus a way to enumerate the full set when generating manifests.
pub trait PermissionEnum: Copy + 'static {
    /// Return the globally stable permission name.
    fn as_str(self) -> &'static str;
    /// Enumerate every permission variant defined by the app.
    fn all() -> &'static [Self];
}

/// Provide access to an app's static [`AppPermissionManifest`].
///
/// Orbital macros and startup registration use this to attach an app's permission declaration to the app's SSR registration metadata.
pub trait AppPermissionManifestProvider {
    /// Return the static manifest for the application.
    fn manifest() -> &'static AppPermissionManifest;
}

/// One concrete permission that can be granted to a user or group.
#[derive(Clone, Copy, Debug)]
pub struct PermissionSpec {
    /// Stable machine-readable permission identifier.
    pub name: &'static str,
    /// Human-friendly explanation shown in permission-management UI.
    pub description: &'static str,
}

/// Group of related permissions owned by one application domain.
///
/// Domains help the UI present large permission sets in manageable sections, such as "Counter Admin" or "Deployment Operations".
#[derive(Clone, Copy, Debug)]
pub struct PermissionDomainSpec {
    /// Stable domain key used internally by synchronization logic.
    pub key: &'static str,
    /// Human-friendly domain title shown in UI.
    pub name: &'static str,
    /// Description of the access area this domain represents.
    pub description: &'static str,
    /// Permissions that belong to this domain.
    pub permissions: &'static [PermissionSpec],
}

/// Full permission declaration for one Orbital application.
///
/// The manifest is the unit Orbital synchronizes at startup. It answers:
///
/// - which app owns the permissions, - how the permissions are grouped into domains, - and what descriptions should appear in management surfaces.
#[derive(Clone, Copy, Debug)]
pub struct AppPermissionManifest {
    /// Owning app id, matching the app registration id.
    pub app_id: &'static str,
    /// Permission domains exposed by the application.
    pub domains: &'static [PermissionDomainSpec],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy)]
    enum SamplePermission {
        View,
        Edit,
    }

    impl PermissionEnum for SamplePermission {
        fn as_str(self) -> &'static str {
            match self {
                Self::View => "sample.view",
                Self::Edit => "sample.edit",
            }
        }

        fn all() -> &'static [Self] {
            &[Self::View, Self::Edit]
        }
    }

    static SAMPLE_PERMISSIONS: &[PermissionSpec] = &[
        PermissionSpec {
            name: "sample.view",
            description: "View sample resources",
        },
        PermissionSpec {
            name: "sample.edit",
            description: "Edit sample resources",
        },
    ];

    static SAMPLE_DOMAINS: &[PermissionDomainSpec] = &[PermissionDomainSpec {
        key: "sample",
        name: "Sample",
        description: "Sample domain",
        permissions: SAMPLE_PERMISSIONS,
    }];

    static SAMPLE_MANIFEST: AppPermissionManifest = AppPermissionManifest {
        app_id: "sample-app",
        domains: SAMPLE_DOMAINS,
    };

    struct SampleManifestProvider;

    impl AppPermissionManifestProvider for SampleManifestProvider {
        fn manifest() -> &'static AppPermissionManifest {
            &SAMPLE_MANIFEST
        }
    }

    #[test]
    fn permission_enum_as_str_and_all_happy_path() {
        assert_eq!(SamplePermission::View.as_str(), "sample.view");
        assert_eq!(SamplePermission::all().len(), 2);
        assert_eq!(
            SamplePermission::all()
                .iter()
                .map(|p| p.as_str())
                .collect::<Vec<_>>(),
            vec!["sample.view", "sample.edit"]
        );
    }

    #[test]
    fn manifest_provider_exposes_domains_and_permissions_happy_path() {
        let m = SampleManifestProvider::manifest();
        assert_eq!(m.app_id, "sample-app");
        assert_eq!(m.domains.len(), 1);
        assert_eq!(m.domains[0].key, "sample");
        assert_eq!(m.domains[0].permissions.len(), 2);
        assert_eq!(m.domains[0].permissions[0].name, "sample.view");
    }

    #[test]
    fn permission_enum_unknown_name_not_in_all_sad() {
        let names: Vec<_> = SamplePermission::all().iter().map(|p| p.as_str()).collect();
        assert!(
            !names.contains(&"sample.delete"),
            "undeclared permission must not appear in manifest enum"
        );
    }
}
