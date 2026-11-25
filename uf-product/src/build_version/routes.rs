//! Server side of the build check: `/_uf/build.json`, `/uf-sw.js` and the
//! marker that tells the shell the routes are mounted.

use std::path::Path;
use std::sync::{Arc, PoisonError, RwLock};

use axum::extract::State;
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use axum::http::HeaderName;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use leptos::config::LeptosOptions;

use super::{is_stamped_build_id, BuildManifest, BUILD_JSON_PATH, SERVICE_WORKER_PATH};

/// The service worker script, served as-is.
const SERVICE_WORKER_JS: &str = include_str!("uf-sw.js");

/// Build id of the last [`build_version_routes`] call; `None` until a host
/// mounts the routes.
static MOUNTED_BUILD: RwLock<Option<String>> = RwLock::new(None);

/// Routes for the build check: `GET /_uf/build.json` (the current build id and
/// its asset list, `no-store`) and `GET /uf-sw.js` (the pkg-cache service
/// worker, scope `/`).
///
/// Reads the pkg dir listing once, here. Merge the router into the host's
/// before `with_state`:
///
/// ```rust,ignore
/// let app = Router::new()
///     .merge(build_version_routes(&leptos_options))
///     .leptos_routes(&leptos_options, routes, App);
/// ```
///
/// Mounting the routes is also what turns the shell's watcher on: until a
/// host calls this, the shell makes no build requests.
pub fn build_version_routes<S: Clone + Send + Sync + 'static>(
    options: &LeptosOptions,
) -> Router<S> {
    let manifest = read_build_manifest(
        Path::new(options.site_root.as_ref()),
        options.site_pkg_dir.as_ref(),
    );
    *MOUNTED_BUILD
        .write()
        .unwrap_or_else(PoisonError::into_inner) = Some(manifest.build.clone());
    router(Arc::new(manifest))
}

fn router<S: Clone + Send + Sync + 'static>(manifest: Arc<BuildManifest>) -> Router<S> {
    Router::new()
        .route(BUILD_JSON_PATH, get(build_json))
        .route(SERVICE_WORKER_PATH, get(service_worker))
        .with_state(manifest)
}

/// List the files under `<site_root>/<pkg_dir>` as `/<pkg_dir>/<file>` paths,
/// sorted. A missing dir gives an empty list.
#[must_use]
pub fn read_build_manifest(site_root: &Path, pkg_dir: &str) -> BuildManifest {
    let pkg_dir = pkg_dir.trim_matches('/');
    let root = site_root.join(pkg_dir);
    let mut assets = Vec::new();
    collect_files(&root, &root, &mut assets);
    let mut assets: Vec<String> = assets
        .into_iter()
        .map(|rel| format!("/{pkg_dir}/{rel}"))
        .collect();
    assets.sort();
    BuildManifest {
        build: pkg_dir.to_string(),
        assets,
    }
}

fn collect_files(root: &Path, dir: &Path, out: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_dir() {
            collect_files(root, &path, out);
        } else if file_type.is_file() {
            let Ok(rel) = path.strip_prefix(root) else {
                continue;
            };
            let parts: Vec<_> = rel
                .components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect();
            out.push(parts.join("/"));
        }
    }
}

async fn build_json(State(manifest): State<Arc<BuildManifest>>) -> impl IntoResponse {
    (
        [(CACHE_CONTROL, "no-store")],
        Json(manifest.as_ref().clone()),
    )
}

async fn service_worker() -> impl IntoResponse {
    (
        [
            (CONTENT_TYPE, "text/javascript"),
            (CACHE_CONTROL, "no-cache"),
            (HeaderName::from_static("service-worker-allowed"), "/"),
        ],
        SERVICE_WORKER_JS,
    )
}

/// What the shell's hidden marker says during SSR: `None` keeps the watcher
/// off, `Some(running_override)` turns it on.
pub(super) fn watch_marker() -> Option<Option<String>> {
    let mounted = MOUNTED_BUILD
        .read()
        .unwrap_or_else(PoisonError::into_inner)
        .clone()?;
    marker_for(&mounted, running_override())
}

fn marker_for(mounted_build: &str, running_override: Option<String>) -> Option<Option<String>> {
    match running_override {
        Some(running) => Some(Some(running)),
        None if is_stamped_build_id(mounted_build) => Some(None),
        None => None,
    }
}

#[cfg(feature = "e2e-build-override")]
fn running_override() -> Option<String> {
    BuildOverride::shared().get()
}

#[cfg(not(feature = "e2e-build-override"))]
fn running_override() -> Option<String> {
    None
}

/// E2e only: makes pages rendered from now on think they run an older build.
///
/// Set it to a stamped id such as `pkg-e2e-old`, and the next page load sees
/// a different build in `/_uf/build.json` and shows the bar, even on a host
/// that serves an unstamped `pkg` dir. Clear it and the page's own build
/// applies again.
#[cfg(feature = "e2e-build-override")]
#[derive(Clone, Debug, Default)]
pub struct BuildOverride(Arc<RwLock<Option<String>>>);

#[cfg(feature = "e2e-build-override")]
impl BuildOverride {
    /// The process-wide override the shell reads.
    #[must_use]
    pub fn shared() -> Self {
        static SHARED: std::sync::OnceLock<BuildOverride> = std::sync::OnceLock::new();
        SHARED.get_or_init(Self::default).clone()
    }

    /// Set or clear the override. Returns `false`, and changes nothing, for an
    /// id that isn't a stamped `pkg-…` name.
    pub fn set(&self, running: Option<String>) -> bool {
        if running
            .as_deref()
            .is_some_and(|id| !is_stamped_build_id(id))
        {
            return false;
        }
        *self.0.write().unwrap_or_else(PoisonError::into_inner) = running;
        true
    }

    /// The current override, if any.
    #[must_use]
    pub fn get(&self) -> Option<String> {
        self.0
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn temp_site(name: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("uf-build-version-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    #[test]
    fn manifest_lists_pkg_files_sorted() {
        let site = temp_site("list");
        let pkg = site.join("pkg-20261125093000-0123456789ab");
        std::fs::create_dir_all(pkg.join("snippets/chunk-1")).expect("dirs");
        for file in [
            "app_bg.wasm",
            "app.js",
            "app.css",
            "snippets/chunk-1/inline0.js",
        ] {
            std::fs::write(pkg.join(file), b"x").expect("write");
        }
        // Files outside the pkg dir stay out.
        std::fs::write(site.join("favicon.ico"), b"x").expect("write");

        let manifest = read_build_manifest(&site, "pkg-20261125093000-0123456789ab");
        assert_eq!(manifest.build, "pkg-20261125093000-0123456789ab");
        assert_eq!(
            manifest.assets,
            vec![
                "/pkg-20261125093000-0123456789ab/app.css",
                "/pkg-20261125093000-0123456789ab/app.js",
                "/pkg-20261125093000-0123456789ab/app_bg.wasm",
                "/pkg-20261125093000-0123456789ab/snippets/chunk-1/inline0.js",
            ]
        );
        let _ = std::fs::remove_dir_all(&site);
    }

    #[test]
    fn manifest_for_missing_dir_is_empty() {
        let site = temp_site("missing");
        let manifest = read_build_manifest(&site, "/pkg/");
        assert_eq!(manifest.build, "pkg");
        assert!(manifest.assets.is_empty());
        let _ = std::fs::remove_dir_all(&site);
    }

    #[tokio::test]
    async fn build_json_serves_manifest_uncached() {
        let manifest = BuildManifest {
            build: "pkg-1".into(),
            assets: vec!["/pkg-1/app.js".into()],
        };
        let app: Router = router(Arc::new(manifest.clone()));
        let res = app
            .oneshot(
                Request::get(BUILD_JSON_PATH)
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.headers()[CACHE_CONTROL], "no-store");
        let body = res.into_body().collect().await.expect("body").to_bytes();
        let parsed: BuildManifest = serde_json::from_slice(&body).expect("json");
        assert_eq!(parsed, manifest);
    }

    #[tokio::test]
    async fn service_worker_headers() {
        let app: Router = router(Arc::new(BuildManifest {
            build: "pkg".into(),
            assets: vec![],
        }));
        let res = app
            .oneshot(
                Request::get(SERVICE_WORKER_PATH)
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.headers()[CONTENT_TYPE], "text/javascript");
        assert_eq!(res.headers()[CACHE_CONTROL], "no-cache");
        assert_eq!(res.headers()["service-worker-allowed"], "/");
        let body = res.into_body().collect().await.expect("body").to_bytes();
        assert!(std::str::from_utf8(&body)
            .expect("utf8")
            .contains("uf-pkg-"));
    }

    #[test]
    fn marker_needs_a_stamped_build_or_an_override() {
        assert_eq!(marker_for("pkg-1", None), Some(None));
        assert_eq!(marker_for("pkg", None), None);
        assert_eq!(
            marker_for("pkg", Some("pkg-e2e-old".into())),
            Some(Some("pkg-e2e-old".into()))
        );
    }

    #[cfg(feature = "e2e-build-override")]
    #[test]
    fn override_accepts_only_stamped_ids() {
        let over = BuildOverride::default();
        assert!(over.set(Some("pkg-e2e-old".into())));
        assert_eq!(over.get().as_deref(), Some("pkg-e2e-old"));
        assert!(!over.set(Some("pkg".into())));
        assert!(!over.set(Some("../etc".into())));
        assert_eq!(over.get().as_deref(), Some("pkg-e2e-old"));
        assert!(over.set(None));
        assert_eq!(over.get(), None);
    }
}
