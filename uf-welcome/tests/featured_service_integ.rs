//! Valence-backed featured catalog integ (System actor + sqlite mem).

#![cfg(feature = "ssr")]
#![allow(missing_docs)]

use std::sync::Arc;

use uf_product::AppRegistration;
use uf_welcome::featured::{add, list, remove, reorder, FeaturedError};
use valence::{
    register_noop_deletion_dispatcher_for_tests, router_key, Actor, InMemoryBackend, Valence,
    MEM_ENGINE_ID,
};

// Second known app so reorder can exercise two registry ids (welcome comes from the crate).
uf_product::inventory::submit! {
    AppRegistration {
        id: "apps",
        name: "Apps",
        description: "Fixture app for featured integ",
        icon: "Cube",
        route_path: "/apps",
        repository: None,
        crate_name: None,
        brand_seed: None,
        permission_manifest: None,
    }
}

fn system_valence() -> Valence {
    register_noop_deletion_dispatcher_for_tests();
    let key = router_key("default", MEM_ENGINE_ID);
    Valence::builder()
        .add_backend("default", Arc::new(InMemoryBackend::new()))
        .default_backend_key(key)
        .with_actor(Actor::System {
            operation: "featured_integ".into(),
        })
        .build()
        .expect("valence")
}

#[tokio::test]
async fn featured_add_list_orders_by_ordinal() {
    let v = system_valence();
    add(&v, "apps", 10).await.expect("add apps");
    add(&v, "welcome", 1).await.expect("add welcome");
    let rows = list(&v).await.expect("list");
    assert_eq!(
        rows.iter().map(|r| r.app_id.as_str()).collect::<Vec<_>>(),
        vec!["welcome", "apps"]
    );
    assert_eq!(rows[0].ordinal, 1);
    assert_eq!(rows[1].ordinal, 10);
}

#[tokio::test]
async fn featured_add_unknown_app() {
    let v = system_valence();
    let err = add(&v, "zz-unknown-app", 0).await.expect_err("unknown");
    assert!(matches!(err, FeaturedError::UnknownApp { .. }));
}

#[tokio::test]
async fn featured_add_duplicate() {
    let v = system_valence();
    add(&v, "welcome", 0).await.expect("add");
    let err = add(&v, "welcome", 1).await.expect_err("dup");
    assert!(matches!(err, FeaturedError::Duplicate { .. }));
}

#[tokio::test]
async fn featured_remove_ok() {
    let v = system_valence();
    add(&v, "welcome", 0).await.expect("add");
    remove(&v, "welcome").await.expect("remove");
    assert!(list(&v).await.expect("list").is_empty());
}

#[tokio::test]
async fn featured_remove_missing() {
    let v = system_valence();
    let err = remove(&v, "welcome").await.expect_err("missing");
    assert!(matches!(err, FeaturedError::NotFound { .. }));
}

#[tokio::test]
async fn featured_reorder_persists_order() {
    let v = system_valence();
    add(&v, "welcome", 0).await.expect("add welcome");
    add(&v, "apps", 1).await.expect("add apps");
    reorder(&v, &["apps".to_string(), "welcome".to_string()])
        .await
        .expect("reorder");
    let rows = list(&v).await.expect("list");
    assert_eq!(rows[0].app_id, "apps");
    assert_eq!(rows[0].ordinal, 0);
    assert_eq!(rows[1].app_id, "welcome");
    assert_eq!(rows[1].ordinal, 1);
}

#[tokio::test]
async fn featured_reorder_missing_err() {
    let v = system_valence();
    add(&v, "welcome", 0).await.expect("add");
    let err = reorder(&v, &["welcome".to_string(), "apps".to_string()])
        .await
        .expect_err("missing apps");
    assert!(matches!(err, FeaturedError::NotFound { .. }));
}
