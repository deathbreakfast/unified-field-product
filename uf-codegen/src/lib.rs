//! Build-time route discovery for Unified Field product apps.
//!
//! Scans workspace member crates (and any `extra_packages`) for `uf_app! { ... }`
//! invocations, then emits generated Rust source that the host `app` crate's `build.rs`
//! includes to compose every discovered app's routes into one shell.
//!
//! Proc-macro expansion of `uf_app!` lives in `uf-product-macros`. Runtime
//! `AppRegistry` / inventory is in `uf-product::routes`. Host `<Routes>` composition
//! and auth gates are host responsibilities.
//!
//! ## Features
//!
//! - **Build-time route discovery** — Scans `src/lib.rs` and `src/main.rs` of every
//!   workspace package for `uf_app!` invocations (via `syn`, without macro expansion)
//!   and writes `uf_registered_routes.rs` plus `uf_app_route_table.rs` under `OUT_DIR`.
//!   [Get started](#getting-started)
//! - **Dedup by app id** — When two packages register the same `app_id`, the `*-app`
//!   package wins over a legacy core crate.
//! - **Generated outputs** — `uf_registered_routes.rs` provides route component
//!   imports; `uf_app_route_table.rs` is a static table for shell page-view analytics.
//!
//! ## Getting started
//!
//! Host product shells call [`generate_registered_routes`] once from the `app` crate's
//! `build.rs` after workspace members declare routes with `uf_app!`. The pass loads
//! Cargo workspace metadata, parses macro invocations as source text, and writes
//! include files your host `include!`s when building the router.
//!
//! **Prerequisites:** add `uf-codegen` as a `build-dependency`; workspace members with
//! `uf_app!` in `src/lib.rs` or `src/main.rs`; host `build.rs` one level under the
//! workspace root so [`RoutesCodegenConfig::workspace_root`] resolves correctly.
//!
//! ```rust,no_run
//! use std::path::PathBuf;
//! use uf_codegen::{generate_registered_routes, RoutesCodegenConfig, RoutesCodegenError};
//!
//! fn main() -> Result<(), RoutesCodegenError> {
//!     // Host build.rs usually sits one level under the workspace root.
//!     let workspace_root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
//!         .parent()
//!         .expect("host crate parent is workspace root")
//!         .to_path_buf();
//!     generate_registered_routes(&RoutesCodegenConfig {
//!         workspace_root,
//!         out_dir: PathBuf::from(std::env::var("OUT_DIR").unwrap()),
//!         extra_packages: vec![],
//!         // List package names to skip; leave empty to scan every member.
//!         excluded_packages: vec![],
//!     })
//! }
//! ```
//!
//! On success the pass returns `Ok(())` and writes `uf_registered_routes.rs` (and
//! `uf_app_route_table.rs`) under `OUT_DIR`. Match [`RoutesCodegenError`] in `build.rs`
//! or propagate with `?`.
//!
//! **Variant — fixture workspace without a full host:** run the named example to emit
//! includes from the bundled `sample-beacon` fixture:
//!
//! ```bash
//! cargo run -p uf-codegen --example emit_routes_table
//! ```
//!
//! Stdout prints `emit_routes_table: OK` and the generated `uf_registered_routes.rs`
//! contains `sample-beacon`.
//!
//! ## Where to look next
//!
//! - [`RoutesCodegenConfig`] — inputs to the codegen pass (`extra_packages`, `excluded_packages`).
//! - [`AppRouteInfo`] — one discovered app's registration metadata.
//! - [`RoutesCodegenError`] — metadata, path, and I/O failures from the pass.
//! - `uf-product-macros` — `uf_app!` fields scanned by this crate.
//! - `uf-product::routes` — runtime registry built from inventory (separate from these includes).
//! - `uf-product/examples` — `app_route_paths` / `auth_shell_host` for runtime discovery smoke.

#![deny(clippy::missing_errors_doc)]

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

mod parser;

/// Failure while discovering `uf_app!` routes or writing generated includes.
///
/// Returned by [`generate_registered_routes`]. Host `build.rs` can match on the
/// variant or print via [`Display`](std::fmt::Display); `?` into `anyhow::Result`
/// also works because this type implements [`std::error::Error`].
#[derive(Debug)]
pub enum RoutesCodegenError {
    /// `cargo_metadata` could not load the workspace manifest.
    Metadata {
        /// Human-readable failure detail (no secrets).
        message: String,
    },
    /// A package manifest path had no parent directory.
    PackagePath {
        /// Manifest path that lacked a parent.
        manifest: String,
        /// Human-readable failure detail.
        message: String,
    },
    /// Reading or writing a file on disk failed.
    Io {
        /// Path that could not be read or written.
        path: PathBuf,
        /// Underlying I/O message.
        message: String,
    },
}

impl std::fmt::Display for RoutesCodegenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Metadata { message } => write!(f, "cargo metadata: {message}"),
            Self::PackagePath { manifest, message } => {
                write!(f, "package path for {manifest}: {message}")
            }
            Self::Io { path, message } => {
                write!(f, "I/O on {}: {message}", path.display())
            }
        }
    }
}

impl std::error::Error for RoutesCodegenError {}

/// Configuration for registered routes generation
pub struct RoutesCodegenConfig {
    /// Root workspace directory (contains Cargo.toml)
    pub workspace_root: PathBuf,
    /// Output directory where generated code will be written
    pub out_dir: PathBuf,
    /// Additional package names to include (beyond workspace members)
    pub extra_packages: Vec<String>,
    /// Workspace package **`name =`** values to skip (e.g. standalone apps not composed into `app`)
    pub excluded_packages: Vec<String>,
}

/// Information about a registered app route
#[derive(Debug, Clone)]
pub struct AppRouteInfo {
    /// App ID (from `uf_app`! id field)
    pub app_id: String,
    /// Display name (from `uf_app`! name field)
    pub app_name: String,
    /// Package name (e.g., "counter-app")
    pub package_name: String,
    /// Routes component name (e.g., "`CounterRoutes`")
    pub routes_component: String,
    /// Route path (e.g., "/counter")
    pub route_path: String,
    /// Optional brand seed override from `uf_app`!
    pub brand_seed: Option<String>,
}

/// Generate route import and analytics table files from discovered `uf_app!` invocations.
///
/// Writes `uf_registered_routes.rs` and `uf_app_route_table.rs` under [`RoutesCodegenConfig::out_dir`].
/// Packages listed in [`RoutesCodegenConfig::excluded_packages`] are skipped. Duplicate `app_id`
/// values keep the `*-app` package when present.
///
/// # Errors
///
/// Returns [`RoutesCodegenError`] when Cargo metadata cannot be loaded, a package path is
/// invalid, or either generated file cannot be written to `out_dir`.
pub fn generate_registered_routes(config: &RoutesCodegenConfig) -> Result<(), RoutesCodegenError> {
    use std::fs;

    // Discover all packages to scan
    let packages = discover_packages(config)?;

    // Parse uf_app! invocations from each package
    let mut app_routes = Vec::new();
    let excluded: std::collections::HashSet<&str> = config
        .excluded_packages
        .iter()
        .map(String::as_str)
        .collect();

    for package_path in packages {
        if let Ok(routes) = parse_package_for_app_routes(&package_path) {
            for route in routes {
                if excluded.contains(route.package_name.as_str()) {
                    continue;
                }
                app_routes.push(route);
            }
        }
    }

    // Deduplicate by app_id. If duplicates exist, prefer `*-app` package names
    // so app-wrapper crates win over legacy/core crates.
    let app_routes = dedupe_app_routes(app_routes);

    // Generate the code
    let generated_code = generate_routes_imports(&app_routes);
    let route_table_code = generate_route_table(&app_routes);

    // Write the generated code
    let dest_path = config.out_dir.join("uf_registered_routes.rs");
    fs::write(&dest_path, generated_code).map_err(|e| RoutesCodegenError::Io {
        path: dest_path.clone(),
        message: e.to_string(),
    })?;
    let table_path = config.out_dir.join("uf_app_route_table.rs");
    fs::write(&table_path, route_table_code).map_err(|e| RoutesCodegenError::Io {
        path: table_path.clone(),
        message: e.to_string(),
    })?;

    // Not `println!`: this is a `cargo:` build-script directive, which must go to stdout
    // for cargo to parse it, so it is exempt from the print_stdout lint.
    #[allow(clippy::print_stdout)]
    {
        println!(
            "cargo:rerun-if-changed={}",
            config.workspace_root.join("Cargo.toml").display()
        );
    }

    Ok(())
}

/// Directory containing a package's manifest (i.e. the package root).
fn package_dir(pkg: &cargo_metadata::Package) -> Result<PathBuf, RoutesCodegenError> {
    pkg.manifest_path
        .parent()
        .map(|dir| dir.as_std_path().to_path_buf())
        .ok_or_else(|| RoutesCodegenError::PackagePath {
            manifest: pkg.manifest_path.to_string(),
            message: "manifest path has no parent directory".to_string(),
        })
}

/// Discover all packages to scan (workspace members + extra packages)
fn discover_packages(config: &RoutesCodegenConfig) -> Result<Vec<PathBuf>, RoutesCodegenError> {
    use cargo_metadata::MetadataCommand;

    // Load cargo metadata
    let manifest_path = config.workspace_root.join("Cargo.toml");
    let metadata = MetadataCommand::new()
        .manifest_path(&manifest_path)
        .exec()
        .map_err(|e| RoutesCodegenError::Metadata {
            message: e.to_string(),
        })?;

    let mut package_paths = Vec::new();

    // Get workspace member packages
    for member_id in &metadata.workspace_members {
        if let Some(pkg) = metadata.packages.iter().find(|p| &p.id == member_id) {
            package_paths.push(package_dir(pkg)?);
        }
    }

    // Add extra packages
    for extra_pkg_name in &config.extra_packages {
        if let Some(pkg) = metadata
            .packages
            .iter()
            .find(|p| p.name.as_str() == extra_pkg_name)
        {
            package_paths.push(package_dir(pkg)?);
        } else {
            // Not `eprintln!`: `cargo:warning=` must be printed on stdout for cargo to
            // parse it as a build-script directive, so it is exempt from print_stdout.
            // Attribute must wrap a block — applying it directly to `println!` is ignored.
            #[allow(clippy::print_stdout)]
            {
                println!("cargo:warning=Extra package '{extra_pkg_name}' not found in workspace");
            }
        }
    }

    Ok(package_paths)
}

/// Directory name of `package_path`, used as a fallback package name when
/// `Cargo.toml` is missing or unparsable.
fn fallback_package_name(package_path: &Path) -> String {
    package_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string()
}

/// Best-effort `name = "..."` extraction from a `Cargo.toml`'s contents, falling back
/// to the package directory name if the field can't be found or parsed.
fn package_name_from_manifest(content: &str, package_path: &Path) -> String {
    content.find("name = \"").map_or_else(
        || fallback_package_name(package_path),
        |start| {
            let start = start + 8; // len of "name = \""
            content[start..].find('"').map_or_else(
                || fallback_package_name(package_path),
                |end| content[start..start + end].to_string(),
            )
        },
    )
}

/// Parse a package directory for `uf_app`! invocations
fn parse_package_for_app_routes(
    package_path: &Path,
) -> Result<Vec<AppRouteInfo>, RoutesCodegenError> {
    let lib_rs = package_path.join("src/lib.rs");
    let main_rs = package_path.join("src/main.rs");

    // Read package name from Cargo.toml
    let cargo_toml = package_path.join("Cargo.toml");
    let package_name = if cargo_toml.exists() {
        let content = std::fs::read_to_string(&cargo_toml).map_err(|e| RoutesCodegenError::Io {
            path: cargo_toml.clone(),
            message: e.to_string(),
        })?;
        // Simple parsing - look for name = "..."
        package_name_from_manifest(&content, package_path)
    } else {
        fallback_package_name(package_path)
    };

    let mut routes = Vec::new();

    // Try parsing lib.rs
    if lib_rs.exists() {
        if let Ok(found) = parser::parse_file_for_app_routes(&lib_rs, &package_name) {
            routes.extend(found);
        }
    }

    // Try parsing main.rs
    if main_rs.exists() {
        if let Ok(found) = parser::parse_file_for_app_routes(&main_rs, &package_name) {
            routes.extend(found);
        }
    }

    Ok(routes)
}

/// Generate the Rust code with route component imports and a macro to include them all
pub(crate) fn generate_routes_imports(app_routes: &[AppRouteInfo]) -> String {
    // Generate code manually to avoid formatting issues with view! macro
    let mut code = String::new();
    code.push_str("// This file is auto-generated by build.rs\n");
    code.push_str("// DO NOT EDIT MANUALLY\n\n");
    code.push_str("#[allow(unused_imports)]\n");
    code.push_str("use leptos::prelude::*;\n");
    code.push_str("// Note: orbital_routes! macro must be imported in the parent module\n\n");

    // Add imports for route components
    for route in app_routes {
        let package_ident = route.package_name.replace('-', "_");
        let _ = writeln!(code, "pub use {package_ident}::{};", route.routes_component);
    }

    code
}

/// Generate static route table for shell page-view analytics (longest-prefix resolution on client).
pub(crate) fn generate_route_table(app_routes: &[AppRouteInfo]) -> String {
    let mut sorted: Vec<_> = app_routes.iter().collect();
    sorted.sort_by_key(|route| std::cmp::Reverse(route.route_path.len()));

    let mut code = String::new();
    code.push_str("// This file is auto-generated by build.rs\n");
    code.push_str("// DO NOT EDIT MANUALLY\n\n");
    code.push_str("pub use uf_product::telemetry::UfAppRouteEntry;\n\n");
    code.push_str("pub static UF_APP_ROUTE_TABLE: &[UfAppRouteEntry] = &[\n");
    for route in sorted {
        let brand_seed = resolve_brand_seed_for_codegen(&route.app_id, route.brand_seed.as_deref());
        let _ = writeln!(
            code,
            "    UfAppRouteEntry {{ app_id: {:?}, app_name: {:?}, route_prefix: {:?}, brand_seed: {:?} }},",
            route.app_id, route.app_name, route.route_path, brand_seed
        );
    }
    code.push_str("];\n");
    code
}

/// Default brand seed colors by product/app id, keyed off the app id prefix.
const PRODUCT_BRAND_SEEDS: &[(&str, &str)] = &[
    ("valence", "#4f6bed"),
    ("chronon", "#eaa300"),
    ("boson", "#7160e8"),
    ("photon", "#00b7c3"),
    ("spectra", "#5c2e91"),
    ("gluon", "#e3008c"),
    ("database", "#5b5fc7"),
    ("permission", "#5b5fc7"),
    ("secrets", "#0b6a0b"),
    ("tag", "#4a89dc"),
    ("counter", "#1a6f94"),
    ("lepton-app", "#4a89dc"),
    ("apps", "#4a89dc"),
    ("auth", "#4a89dc"),
    ("notifications", "#4a89dc"),
    ("welcome", "#4a89dc"),
    ("orbital", "#4a89dc"),
    ("setup-wizard", "#4a89dc"),
    ("marketing", "#4a89dc"),
    ("shell", "#1a6f94"),
];

/// Mirrors `orbital_theme::brand_seed_for_app_id` for build-time route table emission.
pub(crate) fn resolve_brand_seed_for_codegen(app_id: &str, override_seed: Option<&str>) -> String {
    if let Some(seed) = override_seed {
        return seed.to_string();
    }
    PRODUCT_BRAND_SEEDS
        .iter()
        .find(|(id, _)| *id == app_id)
        .map_or_else(|| "#1a6f94".to_string(), |(_, seed)| seed.to_string())
}

/// Prefer `*-app` wrappers when two packages register the same `app_id`.
pub(crate) fn dedupe_app_routes(app_routes: Vec<AppRouteInfo>) -> Vec<AppRouteInfo> {
    let mut deduped = std::collections::BTreeMap::<String, AppRouteInfo>::new();
    for route in app_routes {
        match deduped.get(&route.app_id) {
            None => {
                deduped.insert(route.app_id.clone(), route);
            }
            Some(existing) => {
                let prefer_candidate = route.package_name.ends_with("-app")
                    && !existing.package_name.ends_with("-app");
                if prefer_candidate {
                    deduped.insert(route.app_id.clone(), route);
                }
            }
        }
    }
    deduped.into_values().collect()
}

#[cfg(test)]
mod codegen_contract_tests {
    use super::*;

    fn sample_route(app_id: &str, package: &str, component: &str, path: &str) -> AppRouteInfo {
        AppRouteInfo {
            app_id: app_id.into(),
            app_name: app_id.into(),
            package_name: package.into(),
            routes_component: component.into(),
            route_path: path.into(),
            brand_seed: None,
        }
    }

    #[test]
    fn routes_codegen_error_display_classifies_variants_happy_path() {
        let meta = RoutesCodegenError::Metadata {
            message: "no Cargo.toml".into(),
        };
        assert!(meta.to_string().contains("cargo metadata"));
        let io = RoutesCodegenError::Io {
            path: PathBuf::from("/tmp/out.rs"),
            message: "permission denied".into(),
        };
        assert!(io.to_string().contains("/tmp/out.rs"));
        assert!(io.to_string().contains("permission denied"));
    }

    #[test]
    fn generate_routes_imports_emits_uf_apps_routes_happy_path() {
        let routes = vec![sample_route("apps", "uf-apps", "UfAppsRoutes", "/apps")];
        let code = generate_routes_imports(&routes);
        assert!(
            code.contains("pub use uf_apps::UfAppsRoutes;"),
            "expected UfAppsRoutes import: {code}"
        );
    }

    #[test]
    fn generate_route_table_longest_prefix_first_happy_path() {
        let routes = vec![
            sample_route("apps", "uf-apps", "UfAppsRoutes", "/apps"),
            sample_route("apps-admin", "apps-admin", "AppsAdminRoutes", "/apps/admin"),
        ];
        let code = generate_route_table(&routes);
        let admin_pos = code
            .find("/apps/admin")
            .expect("admin path should be present");
        let apps_pos = code.find("\"/apps\"").expect("apps path should be present");
        assert!(
            admin_pos < apps_pos,
            "longer prefix should sort first:\n{code}"
        );
    }

    #[test]
    fn resolve_brand_seed_override_and_default_happy_path() {
        assert_eq!(
            resolve_brand_seed_for_codegen("apps", Some("#112233")),
            "#112233"
        );
        assert_eq!(resolve_brand_seed_for_codegen("apps", None), "#4a89dc");
    }

    #[test]
    fn resolve_brand_seed_unknown_app_falls_back_sad() {
        assert_eq!(
            resolve_brand_seed_for_codegen("zz-unknown-product", None),
            "#1a6f94"
        );
    }

    #[test]
    fn dedupe_app_routes_prefers_app_wrapper_happy_path() {
        let routes = vec![
            sample_route("counter", "counter-core", "CounterRoutes", "/counter"),
            sample_route("counter", "counter-app", "CounterAppRoutes", "/counter"),
        ];
        let deduped = dedupe_app_routes(routes);
        assert_eq!(deduped.len(), 1);
        assert_eq!(deduped[0].package_name, "counter-app");
        assert_eq!(deduped[0].routes_component, "CounterAppRoutes");
    }

    #[test]
    fn dedupe_app_routes_keeps_first_when_no_app_suffix_sad() {
        let routes = vec![
            sample_route("apps", "uf-apps", "UfAppsRoutes", "/apps"),
            sample_route("apps", "uf-apps-legacy", "LegacyAppsRoutes", "/apps"),
        ];
        let deduped = dedupe_app_routes(routes);
        assert_eq!(deduped.len(), 1);
        assert_eq!(deduped[0].routes_component, "UfAppsRoutes");
    }
}
