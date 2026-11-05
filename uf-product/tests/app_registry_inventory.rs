//! `AppRegistry::auto_discover` enumerates the apps linked into the binary.
#![cfg(feature = "ssr")]
#![allow(missing_docs, clippy::unwrap_used, clippy::expect_used)]

use uf_product::inventory;
use uf_product::routes::{get_all_app_route_paths, AppRegistration, AppRegistry};

inventory::submit! {
    AppRegistration {
        id: "fixture-alpha",
        name: "Fixture Alpha",
        description: "First linked fixture app",
        icon: "Cube",
        route_path: "/fixture-alpha",
        repository: Some("https://example.com/alpha"),
        crate_name: None,
        brand_seed: None,
        permission_manifest: None,
    }
}

inventory::submit! {
    AppRegistration {
        id: "fixture-beta",
        name: "Fixture Beta",
        description: "Second linked fixture app",
        icon: "Cube",
        route_path: "/fixture-beta",
        repository: None,
        crate_name: Some("fixture-beta"),
        brand_seed: Some("#112233"),
        permission_manifest: None,
    }
}

#[test]
fn linked_apps_are_enumerated() {
    let registry = AppRegistry::auto_discover();
    let mut ids: Vec<&str> = registry.iter().map(|app| app.id).collect();
    ids.sort_unstable();
    assert_eq!(ids, ["fixture-alpha", "fixture-beta"]);
    assert_eq!(registry.len(), 2);
}

#[test]
fn every_entry_carries_discovery_metadata() {
    for app in AppRegistry::auto_discover().iter() {
        assert!(!app.id.is_empty());
        assert!(!app.name.is_empty());
        assert!(!app.description.is_empty());
        assert!(!app.icon.is_empty());
        assert!(app.route_path.starts_with('/'), "{}", app.route_path);
    }
}

#[test]
fn lookup_by_id_returns_registered_metadata() {
    let registry = AppRegistry::auto_discover();
    let beta = registry.get("fixture-beta").expect("beta registered");
    assert_eq!(beta.name, "Fixture Beta");
    assert_eq!(beta.route_path, "/fixture-beta");
    assert_eq!(beta.crate_name, Some("fixture-beta"));
    assert_eq!(beta.brand_seed, Some("#112233"));
    let alpha = registry.get("fixture-alpha").expect("alpha registered");
    assert_eq!(alpha.repository, Some("https://example.com/alpha"));
}

#[test]
fn route_paths_list_every_linked_app() {
    let paths = get_all_app_route_paths();
    assert!(paths.contains(&"/fixture-alpha"));
    assert!(paths.contains(&"/fixture-beta"));
}

#[test]
fn unlinked_app_is_not_discovered() {
    let registry = AppRegistry::auto_discover();
    assert!(registry.get("not-linked").is_none());
    assert!(!get_all_app_route_paths().contains(&"/not-linked"));
}
