//! Process-wide SQLite-mem Valence + Higgs factory for Playwright.

use std::sync::{Arc, Mutex, OnceLock};

use higgs::actor_policy::external_actor_json_policy;
use higgs::{HiggsConfig, HiggsValenceFactory};
use lepton_identity::generated::{User, UserStatus, UserUserType};
use valence::{
    register_backend_logical_names, Actor, DatabaseBackend, DatabaseRouter, InMemoryBackend, Model,
    RegisterBackendLogicalNamesOptions, RouterValenceFactory, RouterValenceFactoryConfig,
    SqliteBackend, Valence, ValenceFactory, MEM_ENGINE_ID, SQLITE_ENGINE_ID,
};

use crate::gate_demos::{E2E_UNVERIFIED_SESSION_USER, E2E_VERIFIED_SESSION_USER};

struct E2eState {
    system: Valence,
    router: Arc<DatabaseRouter>,
    higgs: Arc<HiggsConfig>,
    minted_ids: Mutex<Vec<String>>,
}

static E2E_STATE: OnceLock<Arc<E2eState>> = OnceLock::new();

struct SqliteHiggsFactory(RouterValenceFactory);

impl HiggsValenceFactory for SqliteHiggsFactory {
    fn build(&self, actor_json: &serde_json::Value) -> anyhow::Result<Valence> {
        self.0.build(actor_json).map_err(|e| anyhow::anyhow!("{e}"))
    }
}

fn record_id_tail(session_user_id: &str) -> &str {
    session_user_id
        .split_once(':')
        .map(|(_, id)| id)
        .unwrap_or(session_user_id)
}

async fn seed_user(id: &str, valence: &Valence) {
    let now = chrono::Utc::now();
    let user = User::new(
        Some(UserUserType::Person),
        Some("e2e-password-hash".to_string()),
        Some(UserStatus::Active),
        None,
        None,
        Some(now),
        None,
        None,
        now,
        now,
    )
    .expect("build e2e user");
    User::upsert(id, user, valence)
        .await
        .expect("upsert e2e user");
}

/// Build (once) the shared System Valence, router, and Higgs config.
pub async fn init_e2e_valence() -> Arc<Valence> {
    if let Some(existing) = E2E_STATE.get() {
        return Arc::new(existing.system.clone());
    }

    valence::deletion::register_noop_deletion_dispatcher_for_tests();
    if std::env::var_os("VALENCE_OWNERSHIP_UNIFIED_FETCH").is_none() {
        // SAFETY: host boot only; OnceLock reads this before first ownership get.
        unsafe {
            std::env::set_var("VALENCE_OWNERSHIP_UNIFIED_FETCH", "0");
        }
    }

    // Notifications / IndexedDemoItem / User → sqlite:default.
    // WelcomeFeaturedApp schemas target mem:default (ownership bundle on mem).
    let sqlite: Arc<dyn DatabaseBackend> = Arc::new(
        SqliteBackend::connect_memory()
            .await
            .expect("SqliteBackend::connect_memory"),
    );
    let mem: Arc<dyn DatabaseBackend> = Arc::new(InMemoryBackend::new());
    let mut router = DatabaseRouter::new();
    register_backend_logical_names(
        &mut router,
        Arc::clone(&sqlite),
        &["default"],
        RegisterBackendLogicalNamesOptions::default(),
    );
    register_backend_logical_names(
        &mut router,
        mem,
        &["default"],
        RegisterBackendLogicalNamesOptions::default(),
    );
    let router = Arc::new(router);
    let default_key = valence::router_key("default", SQLITE_ENGINE_ID);
    let _mem_key = valence::router_key("default", MEM_ENGINE_ID);

    let system = Valence::builder()
        .database_router(Arc::clone(&router))
        .default_backend_key(default_key.clone())
        .with_actor(Actor::System {
            operation: "e2e_product_host".into(),
        })
        .build()
        .expect("e2e Valence");

    system
        .sync_typed_tables_from_registry()
        .await
        .expect("e2e sync_typed_tables_from_registry");

    seed_user(record_id_tail(E2E_VERIFIED_SESSION_USER), &system).await;
    seed_user(record_id_tail(E2E_UNVERIFIED_SESSION_USER), &system).await;

    let factory: Arc<dyn HiggsValenceFactory> =
        Arc::new(SqliteHiggsFactory(RouterValenceFactory::new(
            Arc::clone(&router),
            RouterValenceFactoryConfig::new(default_key)
                .actor_json_policy(external_actor_json_policy()),
        )));
    let higgs = Arc::new(
        HiggsConfig::builder()
            .valence_factory_arc(factory)
            .build()
            .expect("e2e HiggsConfig"),
    );

    let state = Arc::new(E2eState {
        system: system.clone(),
        router,
        higgs,
        minted_ids: Mutex::new(Vec::new()),
    });
    let _ = E2E_STATE.set(state);
    Arc::new(system)
}

fn state() -> Arc<E2eState> {
    E2E_STATE
        .get()
        .expect("init_e2e_valence must run before e2e accessors")
        .clone()
}

/// Shared System Valence for seed / welcome harness (call [`init_e2e_valence`] at boot).
pub fn e2e_valence() -> Arc<Valence> {
    Arc::new(state().system.clone())
}

/// Shared System Valence for seed mint / delete (never session create).
pub fn e2e_system_valence() -> Valence {
    state().system.clone()
}

/// Shared Valence router for Axum `Extension` (Higgs `host_ctx`).
pub fn e2e_router() -> Arc<DatabaseRouter> {
    Arc::clone(&state().router)
}

/// Process-wide [`HiggsConfig`] for Leptos `provide_context`.
pub fn e2e_higgs_config() -> Arc<HiggsConfig> {
    Arc::clone(&state().higgs)
}

/// Record ids minted by the last seed so the next seed can delete them.
pub fn take_minted_ids() -> Vec<String> {
    std::mem::take(&mut *state().minted_ids.lock().expect("minted_ids"))
}

/// Remember ids created in this seed for isolation on the next seed.
/// Extends the tracked set so prior batches are still deleted on the next wipe.
pub fn store_minted_ids(ids: Vec<String>) {
    state().minted_ids.lock().expect("minted_ids").extend(ids);
}
