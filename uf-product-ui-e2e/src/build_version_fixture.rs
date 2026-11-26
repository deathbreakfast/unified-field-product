//! `POST /api/test/build-override`: set or clear the build the next page load
//! thinks it runs, so the new-version bar can show on a host with an
//! unstamped `pkg` dir.

use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use uf_product::build_version::BuildOverride;

#[derive(Deserialize)]
pub struct BuildOverrideRequest {
    /// A stamped id such as `pkg-e2e-old`, or `null` to clear.
    pub running: Option<String>,
}

pub async fn set_build_override(Json(body): Json<BuildOverrideRequest>) -> StatusCode {
    if BuildOverride::shared().set(body.running) {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::BAD_REQUEST
    }
}
