//! Resolve the usage `viewer_key` for welcome / product shortcuts.

use leptos::prelude::use_context;

/// E2e harness hosts without Higgs may `provide_context` this so usage
/// queries still filter by the seeded viewer id.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UsageViewerOverride(pub String);

/// Tower-session key e2e hosts may set when Higgs is absent (`String` user id).
pub const E2E_USAGE_VIEWER_SESSION_KEY: &str = "uf_e2e_usage_viewer";

/// Resolve the Spectra `viewer_key` for usage aggregators.
///
/// Order: e2e tower-session key → [`UsageViewerOverride`] → Higgs session user id.
/// E2e keys win so harness seeds are not overridden by a partial Higgs context.
pub async fn resolve_usage_viewer_key() -> Option<String> {
    if let Some(v) = e2e_viewer_from_session().await {
        return Some(v);
    }

    if let Some(over) = use_context::<UsageViewerOverride>() {
        return Some(over.0);
    }

    match higgs::Higgs::from_request().await {
        Ok(ctx) => ctx.session_user_id().map(|uid| uid.to_string()),
        Err(_) => None,
    }
}

async fn e2e_viewer_from_session() -> Option<String> {
    use leptos_axum::extract;
    use tower_sessions::Session;

    let session: Session = extract().await.ok()?;
    session
        .get::<String>(E2E_USAGE_VIEWER_SESSION_KEY)
        .await
        .ok()
        .flatten()
}
