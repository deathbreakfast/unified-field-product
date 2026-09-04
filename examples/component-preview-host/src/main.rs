//! Boot Axum + Leptos for the component preview teaching host.
//!
//! Copy this file into a product binary: Axum router, `/pkg` static mount, then
//! `leptos_routes` with [`component_preview_host::shell`]. Pair with `lib.rs`
//! for the `/orbital` catalog mount.

use axum::Router;
use component_preview_host::{shell, App};
use leptos::config::get_configuration;
use leptos_axum::{generate_route_list, LeptosRoutes};
use tower_http::services::ServeDir;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    simple_logger::init_with_level(log::Level::Info).expect("logger");
    any_spawner::Executor::init_futures_executor().expect("futures executor");

    let conf = get_configuration(None).expect("leptos config");
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let site_root = std::path::PathBuf::from(leptos_options.site_root.as_ref());
    let pkg_dir = site_root.join(leptos_options.site_pkg_dir.as_ref());

    let leptos_options_for_routes = leptos_options.clone();
    let leptos_options_state = leptos_options.clone();

    let app = Router::new()
        .nest_service("/pkg", ServeDir::new(pkg_dir))
        .leptos_routes(&leptos_options, routes, move || {
            shell(leptos_options_for_routes.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options_state);

    log::info!("component-preview-host listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("bind component-preview-host");
    axum::serve(listener, app.into_make_service())
        .await
        .expect("serve component-preview-host");
}
