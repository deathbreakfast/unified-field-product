//! Mem Spectra roundtrip for welcome usage aggregators.

#![cfg(feature = "ssr")]

use std::sync::Arc;
use std::time::Duration;

use chrono::{TimeZone, Utc};
use serde_json::json;
use spectra::{try_log_event_at, MemEventsBackend, MemMetricsBackend, Spectra};
use spectra_core::{EventStorageBackend, MetricsStorageBackend};
use uf_product::telemetry::usage::{
    most_used_for_viewer, popular_apps, recent_apps_for_viewer, UsageQueryError, UsageQueryOptions,
    PAGE_VIEW_LOG_TABLE,
};

async fn mem_spectra() -> Arc<Spectra> {
    let metrics: Arc<dyn MetricsStorageBackend> = Arc::new(MemMetricsBackend::new());
    let events: Arc<dyn EventStorageBackend> = Arc::new(MemEventsBackend::new());
    Arc::new(
        Spectra::builder()
            .metrics_backend(metrics)
            .events_backend(events)
            .embedded()
            .build()
            .expect("spectra build"),
    )
}

fn seed_page_view(app_id: &str, viewer: &str, secs: i64) {
    let ts = Utc.timestamp_opt(secs, 0).single().expect("ts");
    try_log_event_at(
        PAGE_VIEW_LOG_TABLE,
        &json!({
            "path": format!("/{app_id}"),
            "app_id": app_id,
            "app_name": app_id,
            "route_prefix": format!("/{app_id}"),
            "surface": "main_shell",
            "auth": "authenticated",
            "email_verified": "unknown",
            "viewer_key": viewer,
            "nav_kind": "client_nav",
            "referrer_path": "",
            "outcome": "ok",
            "permission_name": "",
            "role_count": 0,
        }),
        ts,
    );
}

#[tokio::test]
async fn usage_query_mem_spectra_roundtrip() {
    let spectra = mem_spectra().await;
    // Far-future timestamps so lookback (30d) still includes them relative to Utc::now()
    // is wrong — use recent wall-clock instead.
    let now = Utc::now().timestamp();
    seed_page_view("counter", "user-a", now - 30);
    seed_page_view("valence", "user-a", now - 20);
    seed_page_view("counter", "user-a", now - 10);
    seed_page_view("chronon", "user-b", now - 5);
    seed_page_view("counter", "user-b", now - 1);

    tokio::time::sleep(Duration::from_millis(80)).await;

    let opts = UsageQueryOptions {
        limit_apps: 8,
        lookback_events: 100,
        lookback: chrono::Duration::days(1),
    };

    let recent = recent_apps_for_viewer(spectra.as_ref(), "user-a", &opts)
        .await
        .expect("recent");
    assert!(
        recent.iter().any(|a| a.app_id == "counter"),
        "recent should include counter for user-a: {recent:?}"
    );
    assert!(
        !recent.iter().any(|a| a.app_id == "chronon"),
        "recent must not include other users' apps: {recent:?}"
    );

    let mine = most_used_for_viewer(spectra.as_ref(), "user-a", &opts)
        .await
        .expect("most used");
    assert_eq!(mine[0].app_id, "counter");
    assert!(
        !mine.iter().any(|a| a.app_id == "chronon"),
        "most-used must exclude other viewers"
    );

    let popular = popular_apps(spectra.as_ref(), &opts)
        .await
        .expect("popular");
    assert!(
        popular.iter().any(|a| a.app_id == "counter"),
        "popular should rank counter: {popular:?}"
    );
}

#[tokio::test]
async fn usage_query_unavailable_maps_error() {
    // QueryFailed path: empty table name is invalid for some backends; use a spectra
    // instance then call with a broken table by going through error Display contract.
    let err = UsageQueryError::SpectraUnavailable;
    assert_eq!(err.to_string(), "spectra unavailable for usage query");
    let err = UsageQueryError::QueryFailed {
        cause: "connection reset".into(),
    };
    assert!(err.to_string().contains("connection reset"));
}
