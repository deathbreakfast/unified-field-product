//! K3 / K4 integration for IndexedDemoItem SideEffect + Iter.

use std::sync::Arc;

use valence::{
    Actor, InMemoryBackend, Model, Mutation, MutationKind, RecordId, SchemaRegistry, SideEffect,
    Valence,
};

use super::indexer::{DEMO_APP_ID, DEMO_SOURCE_TABLE, FORCE_SE_ERROR_TITLE_PREFIX};
use super::{IndexedDemoBackfillIter, IndexedDemoIndexer};
use crate::generated::{IndexedDemoItem, UnifiedFieldSearchDocument};
use crate::workspace_search::{document_id, SearchDocumentWriter};

fn mem_valence(actor: Actor) -> Valence {
    let _ = SchemaRegistry::global();
    valence::deletion::register_noop_deletion_dispatcher_for_tests();
    Valence::builder()
        .add_backend("default", Arc::new(InMemoryBackend::new()))
        .with_actor(actor)
        .build()
        .expect("in-memory valence")
}

fn alice() -> Actor {
    Actor::User {
        user_id: "alice".into(),
    }
}

fn bob() -> Actor {
    Actor::User {
        user_id: "bob".into(),
    }
}

fn system() -> Actor {
    Actor::System {
        operation: "indexed_demo_test".into(),
    }
}

async fn create_item(
    v: &Valence,
    user: &str,
    id: &str,
    title: &str,
    link: &str,
) -> IndexedDemoItem {
    let row =
        IndexedDemoItem::new(RecordId::new("user", user), title.into(), link.into()).expect("new");
    IndexedDemoItem::upsert(id, row, v)
        .await
        .expect("upsert source")
}

#[tokio::test]
async fn se_create_indexes_document_happy() {
    let v = mem_valence(alice());
    let _ = create_item(&v, "alice", "d1", "Beacon From SE", "/demo/d1").await;
    let id = document_id(
        &RecordId::new("user", "alice"),
        DEMO_APP_ID,
        DEMO_SOURCE_TABLE,
        "d1",
    );
    let sys = v.with_actor(system());
    let doc = UnifiedFieldSearchDocument::get(&id, &sys)
        .await
        .expect("get")
        .expect("indexed");
    assert_eq!(doc.title(), "Beacon From SE");
    assert_eq!(doc.link(), "/demo/d1");
    assert_eq!(doc.user().id(), "alice");
}

#[tokio::test]
async fn se_update_refreshes_document_happy() {
    let v = mem_valence(alice());
    let _ = create_item(&v, "alice", "d2", "Old Title", "/demo/d2").await;
    let updated = IndexedDemoItem::new(
        RecordId::new("user", "alice"),
        "New Beacon Title".into(),
        "/demo/d2b".into(),
    )
    .expect("new");
    IndexedDemoItem::upsert("d2", updated, &v)
        .await
        .expect("update");
    let id = document_id(
        &RecordId::new("user", "alice"),
        DEMO_APP_ID,
        DEMO_SOURCE_TABLE,
        "d2",
    );
    let sys = v.with_actor(system());
    let doc = UnifiedFieldSearchDocument::get(&id, &sys)
        .await
        .expect("get")
        .expect("indexed");
    assert_eq!(doc.title(), "New Beacon Title");
    assert_eq!(doc.link(), "/demo/d2b");
}

#[tokio::test]
async fn se_delete_removes_document_happy() {
    let v = mem_valence(alice());
    let _ = create_item(&v, "alice", "d3", "Delete Me", "/demo/d3").await;
    let user = RecordId::new("user", "alice");
    let id = document_id(&user, DEMO_APP_ID, DEMO_SOURCE_TABLE, "d3");
    let sys = v.with_actor(system());
    assert!(UnifiedFieldSearchDocument::get(&id, &sys)
        .await
        .expect("get")
        .is_some());

    // Hard-delete source (Model::delete needs deletion dispatcher worker); remove via backend
    // then invoke SE delete path through writer to mirror product delete SE.
    SearchDocumentWriter::delete(&v, &user, DEMO_APP_ID, DEMO_SOURCE_TABLE, "d3")
        .await
        .expect("delete index");
    // Also exercise SE Delete by creating then Model path if possible — use SE directly:
    let before = create_item(&v, "alice", "d3b", "SE Delete", "/demo/d3b").await;
    let field_changes = crate::generated::IndexedDemoItemFieldChanges::compute(Some(&before), None);
    let mutation = Mutation::new(MutationKind::Delete, Some(before), None, field_changes, &v);
    IndexedDemoIndexer
        .on_mutation(&mutation)
        .await
        .expect("se delete");
    let id_b = document_id(&user, DEMO_APP_ID, DEMO_SOURCE_TABLE, "d3b");
    assert!(UnifiedFieldSearchDocument::get(&id_b, &sys)
        .await
        .expect("get")
        .is_none());
}

#[tokio::test]
async fn se_error_does_not_fail_source_write_sad() {
    let v = mem_valence(alice());
    let title = format!("{FORCE_SE_ERROR_TITLE_PREFIX}still-saved");
    let row = IndexedDemoItem::new(
        RecordId::new("user", "alice"),
        title.clone(),
        "/demo/err".into(),
    )
    .expect("new");
    let saved = IndexedDemoItem::upsert("derr", row, &v)
        .await
        .expect("source write must succeed despite SE error");
    assert_eq!(saved.title(), title.as_str());
    let id = document_id(
        &RecordId::new("user", "alice"),
        DEMO_APP_ID,
        DEMO_SOURCE_TABLE,
        "derr",
    );
    let sys = v.with_actor(system());
    assert!(
        UnifiedFieldSearchDocument::get(&id, &sys)
            .await
            .expect("get")
            .is_none(),
        "forced SE error must not leave an index row"
    );
}

#[tokio::test]
async fn se_uses_source_user_field_happy() {
    let v = mem_valence(system());
    let _ = create_item(&v, "bob", "dbob", "Bob Item", "/demo/bob").await;
    let id = document_id(
        &RecordId::new("user", "bob"),
        DEMO_APP_ID,
        DEMO_SOURCE_TABLE,
        "dbob",
    );
    let doc = UnifiedFieldSearchDocument::get(&id, &v)
        .await
        .expect("get")
        .expect("indexed under bob");
    assert_eq!(doc.user().id(), "bob");
    let alice_v = v.with_actor(alice());
    let hits = crate::workspace_search::query(&alice_v, "Bob Item", 10)
        .await
        .expect("query");
    assert!(hits.is_empty(), "alice must not see bob's indexed item");
    let _ = bob(); // keep fixture helper used for clarity
}

#[tokio::test]
async fn iter_backfill_inserts_missing_happy() {
    let v = mem_valence(alice());
    let item = create_item(&v, "alice", "ib1", "Backfill Me", "/demo/ib1").await;
    let user = RecordId::new("user", "alice");
    SearchDocumentWriter::delete(&v, &user, DEMO_APP_ID, DEMO_SOURCE_TABLE, "ib1")
        .await
        .expect("strip index");
    let sys = v.with_actor(system());
    let id = document_id(&user, DEMO_APP_ID, DEMO_SOURCE_TABLE, "ib1");
    assert!(UnifiedFieldSearchDocument::get(&id, &sys)
        .await
        .expect("get")
        .is_none());

    let eval = IndexedDemoBackfillIter
        .should_run(&item, &v)
        .await
        .expect("should_run");
    assert!(eval.should_run, "expected run: {}", eval.reason);
    IndexedDemoBackfillIter
        .execute(&item, &v)
        .await
        .expect("execute");
    let doc = UnifiedFieldSearchDocument::get(&id, &sys)
        .await
        .expect("get")
        .expect("backfilled");
    assert_eq!(doc.title(), "Backfill Me");
}

#[tokio::test]
async fn iter_skips_fresh_sad() {
    let v = mem_valence(alice());
    let item = create_item(&v, "alice", "ib2", "Already Fresh", "/demo/ib2").await;
    let eval = IndexedDemoBackfillIter
        .should_run(&item, &v)
        .await
        .expect("should_run");
    assert!(!eval.should_run, "expected skip: {}", eval.reason);
}

#[tokio::test]
async fn iter_backfill_preserves_owner_happy() {
    let v = mem_valence(system());
    let item = create_item(&v, "bob", "ib3", "Owner Preserve", "/demo/ib3").await;
    let user = RecordId::new("user", "bob");
    SearchDocumentWriter::delete(&v, &user, DEMO_APP_ID, DEMO_SOURCE_TABLE, "ib3")
        .await
        .expect("strip");
    IndexedDemoBackfillIter
        .execute(&item, &v)
        .await
        .expect("backfill");
    let id = document_id(&user, DEMO_APP_ID, DEMO_SOURCE_TABLE, "ib3");
    let doc = UnifiedFieldSearchDocument::get(&id, &v)
        .await
        .expect("get")
        .expect("doc");
    assert_eq!(doc.user().id(), "bob");
}
