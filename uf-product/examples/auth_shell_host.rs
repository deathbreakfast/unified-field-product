//! Auth shell host: register two sample apps, gate discovery behind a session.
//!
//! This is an **Axum** oneshot demo (inventory + HTTP middleware). It is not
//! Leptos `RequireAuthenticated` — use that gate in product UI hosts.
//!
//! ## When to use
//! Smoke `AppRegistration` inventory + protected shell route discovery.
//!
//! ## Command
//! ```bash
//! export CARGO_BUILD_JOBS=1
//! export CARGO_TARGET_DIR=target-uf-product
//! cargo run -p uf-product --example auth_shell_host --features ssr
//! ```
//!
//! ## Success
//! Stdout prints `auth_shell_host: OK — /shell deny/allow + two registered apps`.
//!
//! ## Look next
//! Compose `RequireAuthenticated` around product routes; register apps with `uf_app!`
//! (`uf_app_registration` example / `shell-chrome-host`).

#![allow(clippy::print_stdout, clippy::unwrap_used, clippy::expect_used)]

use axum::body::Body;
use axum::extract::Extension;
use axum::http::{Request, StatusCode};
use axum::middleware::{from_fn, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use http_body_util::BodyExt;
use tower::ServiceExt;
use uf_product::routes::get_all_app_route_paths;
use uf_product::{inventory, AppRegistration};

inventory::submit! {
    AppRegistration {
        id: "sample-alpha",
        name: "Sample Alpha",
        description: "First sample app registered by the auth shell host.",
        icon: "Cube",
        route_path: "/sample-alpha",
        repository: None,
        crate_name: None,
        brand_seed: None,
        permission_manifest: None,
    }
}

inventory::submit! {
    AppRegistration {
        id: "sample-beta",
        name: "Sample Beta",
        description: "Second sample app registered by the auth shell host.",
        icon: "Apps",
        route_path: "/sample-beta",
        repository: None,
        crate_name: None,
        brand_seed: None,
        permission_manifest: None,
    }
}

#[derive(Clone)]
struct DemoSession {
    user_id: String,
}

async fn require_session(req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    if req.extensions().get::<DemoSession>().is_some() {
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

async fn inject_demo_session(mut req: Request<Body>, next: Next) -> Response {
    if let Some(user) = req
        .headers()
        .get("x-demo-user")
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
    {
        req.extensions_mut().insert(DemoSession { user_id: user });
    }
    next.run(req).await
}

async fn shell_api(Extension(session): Extension<DemoSession>) -> impl IntoResponse {
    let paths = get_all_app_route_paths();
    Json(serde_json::json!({
        "path": "/shell",
        "user": session.user_id,
        "apps": paths,
    }))
}

fn app() -> Router {
    Router::new()
        .route("/shell", get(shell_api))
        .route_layer(from_fn(require_session))
        .layer(from_fn(inject_demo_session))
}

#[tokio::main]
async fn main() {
    let paths = get_all_app_route_paths();
    assert!(
        paths.contains(&"/sample-alpha") && paths.contains(&"/sample-beta"),
        "expected both sample apps, got {paths:?}"
    );

    let denied = app()
        .oneshot(
            Request::builder()
                .uri("/shell")
                .body(Body::empty())
                .expect("req"),
        )
        .await
        .expect("oneshot")
        .status();
    assert_eq!(denied, StatusCode::UNAUTHORIZED);

    let response = app()
        .oneshot(
            Request::builder()
                .uri("/shell")
                .header("x-demo-user", "demo-ops")
                .body(Body::empty())
                .expect("req"),
        )
        .await
        .expect("oneshot");
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
    assert_eq!(body["path"], "/shell");
    let apps = body["apps"].as_array().expect("apps");
    assert!(apps.iter().any(|p| p == "/sample-alpha"));
    assert!(apps.iter().any(|p| p == "/sample-beta"));

    println!("auth_shell_host: OK — /shell deny/allow + two registered apps");
}
