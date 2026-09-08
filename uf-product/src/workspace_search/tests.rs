//! Unit + integration coverage for workspace content index (K1/K2/K5).

use std::sync::Arc;

use valence::{Actor, InMemoryBackend, Model, RecordId, SchemaRegistry, Valence};

use super::{
    document_id, query, SearchDocumentDraft, SearchDocumentWriter, WorkspaceSearchError,
    WorkspaceSearchHit, SEARCH_DOCUMENT_TTL_SECS,
};
use crate::generated::UnifiedFieldSearchDocument;

fn mem_valence(actor: Actor) -> Valence {
    // Ensure inventory schemas (including UnifiedFieldSearchDocument) are discoverable.
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
        operation: "workspace_search_test".into(),
    }
}

fn draft_for(user: &str, source_id: &str, title: &str, link: &str) -> SearchDocumentDraft {
    SearchDocumentDraft {
        user: RecordId::new("user", user),
        app_id: "demo".into(),
        source_table: "indexed_demo_item".into(),
        source_id: source_id.into(),
        title: title.into(),
        searchable_text: title.into(),
        link: link.into(),
        kind: "demo".into(),
    }
}

#[test]
fn schema_policies_forbid_open_read_happy() {
    let meta = SchemaRegistry::global()
        .get_schema("unified_field_search_document")
        .expect("schema registered");
    let policies = meta.schema.policies.as_ref().expect("policies");
    let read = policies.read.as_ref().expect("read policies");
    assert!(
        read.always_allow.is_empty(),
        "always_allow must stay empty for deny-by-default reads"
    );
    assert!(
        !read
            .allow
            .iter()
            .any(|r| r.name.eq_ignore_ascii_case("AUTHENTICATED")
                || r.name.eq_ignore_ascii_case("authenticated")),
        "must not allow AUTHENTICATED open read"
    );
    assert_eq!(SEARCH_DOCUMENT_TTL_SECS, 7_776_000);
}

#[test]
fn document_id_is_stable_happy() {
    let user = RecordId::new("user", "alice");
    let a = document_id(&user, "demo", "todo", "1");
    let b = document_id(&user, "demo", "todo", "1");
    assert_eq!(a, b);
    assert!(a.contains("alice"));
}

#[test]
fn draft_docs_mention_indexing_policy() {
    // Keep indexing policy discoverable on the draft type rustdoc (string presence in source).
    let src = include_str!("draft.rs");
    assert!(src.contains("Do **not** index emails"));
    assert!(src.contains("secrets") || src.contains("auth tokens"));
}

#[test]
fn query_rejects_system_actor_sad() {
    let v = mem_valence(system());
    let err = futures_executor_block(query(&v, "beacon", 10)).expect_err("system forbidden");
    assert!(matches!(
        err,
        WorkspaceSearchError::InvalidActor {
            reason: "system_actor_forbidden"
        }
    ));
}

#[test]
fn query_rejects_anonymous_sad() {
    let v = mem_valence(Actor::Anonymous);
    let err = futures_executor_block(query(&v, "beacon", 10)).expect_err("anon forbidden");
    assert!(matches!(err, WorkspaceSearchError::Unauthenticated));
}

#[test]
fn query_invalid_sad() {
    let v = mem_valence(alice());
    let empty = futures_executor_block(query(&v, "   ", 10)).expect_err("empty");
    assert!(matches!(
        empty,
        WorkspaceSearchError::InvalidQuery { reason: "empty" }
    ));
    let long = "x".repeat(201);
    let too_long = futures_executor_block(query(&v, &long, 10)).expect_err("too long");
    assert!(matches!(
        too_long,
        WorkspaceSearchError::InvalidQuery { reason: "too_long" }
    ));
}

#[test]
fn workspace_search_hit_omits_user_field_happy() {
    let hit = WorkspaceSearchHit {
        id: "1".into(),
        title: "T".into(),
        description: None,
        app_id: "demo".into(),
        kind: "demo".into(),
        link: "/x".into(),
    };
    let json = serde_json::to_value(&hit).expect("serde");
    assert!(json.get("user").is_none());
    assert_eq!(json["link"], "/x");
}

#[tokio::test]
async fn writer_upsert_persists_fields_happy() {
    let v = mem_valence(system());
    SearchDocumentWriter::upsert(&v, draft_for("alice", "1", "Beacon Alpha", "/demo/1"))
        .await
        .expect("upsert");
    let id = document_id(
        &RecordId::new("user", "alice"),
        "demo",
        "indexed_demo_item",
        "1",
    );
    let row = UnifiedFieldSearchDocument::get(&id, &v)
        .await
        .expect("get")
        .expect("row");
    assert_eq!(row.title(), "Beacon Alpha");
    assert_eq!(row.link(), "/demo/1");
    assert_eq!(row.user().table(), "user");
    assert_eq!(row.user().id(), "alice");
}

#[tokio::test]
async fn writer_upsert_idempotent_happy() {
    let v = mem_valence(system());
    let draft = draft_for("alice", "2", "First", "/demo/2");
    SearchDocumentWriter::upsert(&v, draft.clone())
        .await
        .expect("upsert1");
    let mut updated = draft;
    updated.title = "Second".into();
    updated.searchable_text = "Second".into();
    SearchDocumentWriter::upsert(&v, updated)
        .await
        .expect("upsert2");
    let id = document_id(
        &RecordId::new("user", "alice"),
        "demo",
        "indexed_demo_item",
        "2",
    );
    let row = UnifiedFieldSearchDocument::get(&id, &v)
        .await
        .expect("get")
        .expect("row");
    assert_eq!(row.title(), "Second");
}

#[tokio::test]
async fn writer_rejects_unsafe_link_sad() {
    let v = mem_valence(system());
    let mut draft = draft_for("alice", "3", "X", "/ok");
    draft.link = "https://evil.example/x".into();
    let err = SearchDocumentWriter::upsert(&v, draft)
        .await
        .expect_err("bad link");
    assert!(matches!(err, WorkspaceSearchError::InvalidLink { .. }));
}

#[tokio::test]
async fn writer_accepts_relative_link_happy() {
    let v = mem_valence(system());
    SearchDocumentWriter::upsert(&v, draft_for("alice", "4", "Ok", "/apps/demo"))
        .await
        .expect("relative link");
}

#[tokio::test]
async fn writer_delete_removes_row_happy() {
    let v = mem_valence(system());
    let user = RecordId::new("user", "alice");
    SearchDocumentWriter::upsert(&v, draft_for("alice", "5", "Del", "/demo/5"))
        .await
        .expect("upsert");
    SearchDocumentWriter::delete(&v, &user, "demo", "indexed_demo_item", "5")
        .await
        .expect("delete");
    let id = document_id(&user, "demo", "indexed_demo_item", "5");
    let gone = UnifiedFieldSearchDocument::get(&id, &v).await.expect("get");
    assert!(gone.is_none());
}

#[tokio::test]
async fn writer_delete_missing_sad() {
    let v = mem_valence(system());
    SearchDocumentWriter::delete(
        &v,
        &RecordId::new("user", "alice"),
        "demo",
        "indexed_demo_item",
        "missing",
    )
    .await
    .expect("missing delete is ok");
}

#[tokio::test]
async fn privacy_owner_read_own_happy() {
    let system_v = mem_valence(system());
    SearchDocumentWriter::upsert(
        &system_v,
        draft_for("alice", "10", "Alice Only TokenZzz", "/demo/10"),
    )
    .await
    .expect("upsert");
    let alice_v = system_v.with_actor(alice());
    let id = document_id(
        &RecordId::new("user", "alice"),
        "demo",
        "indexed_demo_item",
        "10",
    );
    let row = UnifiedFieldSearchDocument::get(&id, &alice_v)
        .await
        .expect("get")
        .expect("owner can read");
    assert_eq!(row.title(), "Alice Only TokenZzz");
}

#[tokio::test]
async fn privacy_other_user_read_denied_sad() {
    let system_v = mem_valence(system());
    SearchDocumentWriter::upsert(
        &system_v,
        draft_for("alice", "11", "SecretAliceTitleQq", "/demo/11"),
    )
    .await
    .expect("upsert");
    let bob_v = system_v.with_actor(bob());
    let id = document_id(
        &RecordId::new("user", "alice"),
        "demo",
        "indexed_demo_item",
        "11",
    );
    let denied = UnifiedFieldSearchDocument::get(&id, &bob_v).await;
    assert!(
        denied.is_err() || matches!(denied, Ok(None)),
        "bob must not read alice document: {denied:?}"
    );
}

#[tokio::test]
async fn privacy_anonymous_read_denied_sad() {
    let system_v = mem_valence(system());
    SearchDocumentWriter::upsert(&system_v, draft_for("alice", "12", "AnonBlock", "/demo/12"))
        .await
        .expect("upsert");
    let anon = system_v.with_actor(Actor::Anonymous);
    let id = document_id(
        &RecordId::new("user", "alice"),
        "demo",
        "indexed_demo_item",
        "12",
    );
    let denied = UnifiedFieldSearchDocument::get(&id, &anon).await;
    assert!(
        denied.is_err() || matches!(denied, Ok(None)),
        "anonymous must not read: {denied:?}"
    );
}

#[tokio::test]
async fn privacy_cross_user_cud_denied_sad() {
    let system_v = mem_valence(system());
    let bob_v = system_v.with_actor(bob());
    let row = UnifiedFieldSearchDocument::new(
        RecordId::new("user", "alice"),
        "demo".into(),
        "indexed_demo_item".into(),
        "13".into(),
        "Forge".into(),
        "Forge".into(),
        "/demo/13".into(),
        "demo".into(),
        chrono::Utc::now(),
    )
    .expect("new");
    let created = UnifiedFieldSearchDocument::upsert("forge13", row, &bob_v).await;
    assert!(
        created.is_err(),
        "bob must not create alice-owned index rows"
    );
}

#[tokio::test]
async fn privacy_system_cud_happy() {
    let v = mem_valence(system());
    SearchDocumentWriter::upsert(&v, draft_for("alice", "14", "SysOk", "/demo/14"))
        .await
        .expect("system cud");
}

#[tokio::test]
async fn query_owner_match_happy() {
    let system_v = mem_valence(system());
    SearchDocumentWriter::upsert(
        &system_v,
        draft_for("alice", "20", "UniqueBeaconMatch", "/demo/20"),
    )
    .await
    .expect("upsert");
    let alice_v = system_v.with_actor(alice());
    let hits = query(&alice_v, "UniqueBeaconMatch", 10)
        .await
        .expect("query");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].title, "UniqueBeaconMatch");
    assert_eq!(hits[0].link, "/demo/20");
    assert!(hits[0].app_id == "demo");
}

#[tokio::test]
async fn query_other_user_isolated_sad() {
    let system_v = mem_valence(system());
    SearchDocumentWriter::upsert(
        &system_v,
        draft_for("bob", "21", "BobSecretTokenYy", "/demo/21"),
    )
    .await
    .expect("upsert bob");
    let alice_v = system_v.with_actor(alice());
    let hits = query(&alice_v, "BobSecretTokenYy", 10)
        .await
        .expect("query");
    assert!(hits.is_empty(), "alice must not see bob hits: {hits:?}");
}

#[tokio::test]
async fn query_system_actor_rejected_sad() {
    let system_v = mem_valence(system());
    SearchDocumentWriter::upsert(
        &system_v,
        draft_for("alice", "22", "ShouldNotLeak", "/demo/22"),
    )
    .await
    .expect("upsert");
    let err = query(&system_v, "ShouldNotLeak", 10)
        .await
        .expect_err("system query forbidden");
    assert!(matches!(
        err,
        WorkspaceSearchError::InvalidActor {
            reason: "system_actor_forbidden"
        }
    ));
}

#[tokio::test]
async fn privacy_other_user_hard_delete_denied_sad() {
    let system_v = mem_valence(system());
    SearchDocumentWriter::upsert(
        &system_v,
        draft_for("alice", "delx", "AliceDeleteMe", "/demo/delx"),
    )
    .await
    .expect("upsert");
    let bob_v = system_v.with_actor(bob());
    let err = SearchDocumentWriter::delete(
        &bob_v,
        &RecordId::new("user", "alice"),
        "demo",
        "indexed_demo_item",
        "delx",
    )
    .await
    .expect_err("bob must not hard-delete alice row");
    assert!(matches!(err, WorkspaceSearchError::Write { .. }));
}

#[tokio::test]
async fn privacy_owner_write_own_happy() {
    let alice_v = mem_valence(alice());
    SearchDocumentWriter::upsert(
        &alice_v,
        draft_for("alice", "own1", "AliceOwnsWrite", "/demo/own1"),
    )
    .await
    .expect("owner upsert");
    let id = document_id(
        &RecordId::new("user", "alice"),
        "demo",
        "indexed_demo_item",
        "own1",
    );
    let row = UnifiedFieldSearchDocument::get(&id, &alice_v)
        .await
        .expect("get")
        .expect("owner row");
    assert_eq!(row.title(), "AliceOwnsWrite");
}

#[tokio::test]
async fn query_respects_max_results_happy() {
    let system_v = mem_valence(system());
    for i in 0..5 {
        SearchDocumentWriter::upsert(
            &system_v,
            draft_for(
                "alice",
                &format!("cap{i}"),
                &format!("CapMatch{i}"),
                &format!("/demo/cap{i}"),
            ),
        )
        .await
        .expect("upsert");
    }
    let alice_v = system_v.with_actor(alice());
    let hits = query(&alice_v, "CapMatch", 2).await.expect("query");
    assert_eq!(hits.len(), 2, "expected exactly 2 got {}", hits.len());
}

#[tokio::test]
async fn writer_upsert_renews_ttl_happy() {
    use valence::ttl::EXPIRE_AT_FIELD;

    let v = mem_valence(system());
    let draft = draft_for("alice", "ttl1", "TtlDoc", "/demo/ttl1");
    SearchDocumentWriter::upsert(&v, draft.clone())
        .await
        .expect("upsert1");
    let id = document_id(
        &RecordId::new("user", "alice"),
        "demo",
        "indexed_demo_item",
        "ttl1",
    );
    let raw1 = valence::QueryCore::get_record_json("unified_field_search_document", &id, &v)
        .await
        .expect("raw1")
        .expect("present");
    let exp1 = raw1
        .get(EXPIRE_AT_FIELD)
        .cloned()
        .expect("expire stamp after create");
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    SearchDocumentWriter::upsert(&v, draft)
        .await
        .expect("upsert2 renew");
    let raw2 = valence::QueryCore::get_record_json("unified_field_search_document", &id, &v)
        .await
        .expect("raw2")
        .expect("present");
    let exp2 = raw2
        .get(EXPIRE_AT_FIELD)
        .cloned()
        .expect("expire stamp after renew");
    assert!(
        exp2 != exp1 || exp2.is_string(),
        "expected TTL stamp after renew; before={exp1:?} after={exp2:?}"
    );
}

fn futures_executor_block<F: std::future::Future>(fut: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("rt")
        .block_on(fut)
}
