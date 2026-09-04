//! Higgs server functions for Help visits, repository lookup, and report submit.
//!
//! All public `Result<_, ServerFnError>` functions validate inputs where noted,
//! then map [`crate::HelpError`] through [`HelpError::into_server_fn_error`]
//! (message string only on the client). Without the `ssr` feature, each call
//! returns `ServerFnError` with message `"SSR required"`.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::service::{HelpStepKey, HelpVisitRecord};

/// Pending step identity returned to the client (bodies stay in inventory).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelpPendingStepDto {
    /// Route path.
    pub route: String,
    /// Feature highlight key.
    pub feature_highlight: String,
    /// Optional spotlight DOM id.
    pub spotlight: Option<String>,
    /// Inventory sort order.
    pub order: u16,
}

#[cfg_attr(not(feature = "ssr"), allow(dead_code))]
fn validate_route(route: &str) -> Result<(), crate::HelpError> {
    if route.is_empty() || route.len() > 512 {
        return Err(crate::HelpError::InvalidRoute("empty or oversized"));
    }
    Ok(())
}

#[cfg_attr(not(feature = "ssr"), allow(dead_code))]
fn validate_highlight(feature_highlight: &str) -> Result<(), crate::HelpError> {
    if feature_highlight.is_empty() || feature_highlight.len() > 256 {
        return Err(crate::HelpError::InvalidHighlight("empty or oversized"));
    }
    Ok(())
}

/// List Valence visit rows for the signed-in user on `route`.
///
/// # Errors
///
/// - [`crate::HelpError::InvalidRoute`] when `route` is empty or longer than 512 bytes.
/// - [`crate::HelpError::Unauthenticated`] when there is no session.
/// - [`crate::HelpError::Storage`] on Valence I/O failure.
/// - `"SSR required"` when this crate is built without the `ssr` feature.
#[uf_product_macros::server]
pub async fn help_list_visits_for_route(
    route: String,
) -> Result<Vec<HelpVisitRecord>, ServerFnError> {
    validate_route(&route)?;
    #[cfg(feature = "ssr")]
    {
        list_visits_ssr(&route)
            .await
            .map_err(crate::HelpError::into_server_fn_error)
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = route;
        Err(ServerFnError::new("SSR required"))
    }
}

/// Pending inventory steps for `route` for the signed-in user.
///
/// Anonymous clients should use local storage + [`crate::compute_pending`] instead.
///
/// # Errors
///
/// Same validation and session/storage failures as [`help_list_visits_for_route`].
#[uf_product_macros::server]
pub async fn help_pending_steps_for_route(
    route: String,
) -> Result<Vec<HelpPendingStepDto>, ServerFnError> {
    validate_route(&route)?;
    #[cfg(feature = "ssr")]
    {
        let visits = list_visits_ssr(&route).await?;
        let inventory = crate::collect_help_steps_for_route(&route);
        let pending = crate::service::compute_pending(&inventory, &visits);
        Ok(pending
            .into_iter()
            .map(|d| HelpPendingStepDto {
                route: d.route.to_string(),
                feature_highlight: d.feature_highlight.to_string(),
                spotlight: d.spotlight.map(str::to_string),
                order: d.order,
            })
            .collect())
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = route;
        Err(ServerFnError::new("SSR required"))
    }
}

/// Mark steps seen (`replay = false`) for the signed-in user.
///
/// Also merges any provided `local_visits` that are missing server-side.
///
/// # Errors
///
/// - [`crate::HelpError::InvalidRoute`] / [`crate::HelpError::InvalidHighlight`] per step key.
/// - [`crate::HelpError::Unauthenticated`] when there is no session.
/// - [`crate::HelpError::Storage`] on Valence upsert failure.
/// - `"SSR required"` without the `ssr` feature.
#[uf_product_macros::server]
pub async fn help_mark_steps_seen(
    steps: Vec<HelpStepKey>,
    local_visits: Vec<HelpVisitRecord>,
) -> Result<(), ServerFnError> {
    for step in &steps {
        validate_route(&step.route)?;
        validate_highlight(&step.feature_highlight)?;
    }
    #[cfg(feature = "ssr")]
    {
        mark_steps_seen_ssr(&steps, &local_visits)
            .await
            .map_err(crate::HelpError::into_server_fn_error)
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = (steps, local_visits);
        Err(ServerFnError::new("SSR required"))
    }
}

/// Set `replay = true` for visit rows on `route` only (signed-in).
///
/// # Errors
///
/// - [`crate::HelpError::InvalidRoute`] when `route` fails validation.
/// - [`crate::HelpError::Unauthenticated`] when there is no session.
/// - [`crate::HelpError::Storage`] on Valence read/write failure.
/// - `"SSR required"` without the `ssr` feature.
#[uf_product_macros::server]
pub async fn help_request_replay_for_route(route: String) -> Result<(), ServerFnError> {
    validate_route(&route)?;
    #[cfg(feature = "ssr")]
    {
        request_replay_ssr(&route)
            .await
            .map_err(crate::HelpError::into_server_fn_error)
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = route;
        Err(ServerFnError::new("SSR required"))
    }
}

/// Resolve `AppRegistration.repository` for the given route on the server.
///
/// # Errors
///
/// - [`crate::HelpError::InvalidRoute`] when `route` fails validation.
/// - `"SSR required"` without the `ssr` feature.
#[uf_product_macros::server]
pub async fn help_repository_for_route(route: String) -> Result<Option<String>, ServerFnError> {
    validate_route(&route)?;
    #[cfg(feature = "ssr")]
    {
        Ok(crate::repository::resolve_help_repository(&route).map(str::to_string))
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = route;
        Err(ServerFnError::new("SSR required"))
    }
}

/// Submit a bug report via the GitHub issues bot.
///
/// # Errors
///
/// - [`crate::HelpError::Validation`] for empty or oversized fields.
/// - [`crate::HelpError::RateLimited`] when the host throttle fires.
/// - [`crate::HelpError::Misconfigured`] when the route has no parseable GitHub repository.
/// - [`crate::HelpError::GitHubUpstream`] on GitHub API failure.
/// - `"SSR required"` without the `ssr` feature.
#[allow(clippy::too_many_arguments)]
#[uf_product_macros::server]
pub async fn submit_help_bug_report(
    route: String,
    title: String,
    description: String,
    steps: String,
    expected: String,
    actual: String,
    app_version: Option<String>,
    browser_os: Option<String>,
    contact_email: Option<String>,
) -> Result<String, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::service_reports::submit_bug(
            &route,
            crate::github::BugReportPayload {
                title,
                description,
                steps,
                expected,
                actual,
                app_version,
                browser_os,
                contact_email,
            },
        )
        .await
        .map_err(crate::HelpError::into_server_fn_error)
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = (
            route,
            title,
            description,
            steps,
            expected,
            actual,
            app_version,
            browser_os,
            contact_email,
        );
        Err(ServerFnError::new("SSR required"))
    }
}

/// Submit a feature request via the GitHub issues bot.
///
/// # Errors
///
/// Same failure classes as [`submit_help_bug_report`] (validation, rate limit,
/// misconfigured repository, GitHub upstream, SSR gate).
#[uf_product_macros::server]
pub async fn submit_help_feature_request(
    route: String,
    title: String,
    problem: String,
    proposed: String,
    alternatives: Option<String>,
    contact_email: Option<String>,
) -> Result<String, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::service_reports::submit_feature(
            &route,
            crate::github::FeatureRequestPayload {
                title,
                problem,
                proposed,
                alternatives,
                contact_email,
            },
        )
        .await
        .map_err(crate::HelpError::into_server_fn_error)
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = (route, title, problem, proposed, alternatives, contact_email);
        Err(ServerFnError::new("SSR required"))
    }
}

/// Submit a security report via the private vulnerability channel (never public issues).
///
/// # Errors
///
/// Same failure classes as [`submit_help_bug_report`] (validation, rate limit,
/// misconfigured repository, GitHub upstream, SSR gate).
#[uf_product_macros::server]
pub async fn submit_help_security_report(
    route: String,
    summary: String,
    description: String,
    repro: String,
    affected: String,
    severity: Option<String>,
    contact_email: Option<String>,
) -> Result<(), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        crate::service_reports::submit_security(
            &route,
            crate::github::SecurityReportPayload {
                summary,
                description,
                repro,
                affected,
                severity,
                contact_email,
            },
        )
        .await
        .map_err(crate::HelpError::into_server_fn_error)
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = (
            route,
            summary,
            description,
            repro,
            affected,
            severity,
            contact_email,
        );
        Err(ServerFnError::new("SSR required"))
    }
}

#[cfg(feature = "ssr")]
async fn list_visits_ssr(pathname: &str) -> Result<Vec<HelpVisitRecord>, crate::HelpError> {
    use uf_product::generated::HelpTourStepVisit;
    use valence::{RecordPredicate, StringPredicate};

    let ctx = uf_product::ssr::higgs()
        .await
        .map_err(|e| crate::HelpError::Storage(e.to_string()))?;
    let user_id_str = ctx
        .session_user_id()
        .ok_or(crate::HelpError::Unauthenticated("list_visits"))?
        .clone();
    let user = uf_product::ssr::session_user_record_id(&user_id_str)
        .map_err(|e| crate::HelpError::Storage(e.to_string()))?;
    let v = uf_product::ssr::valence(&ctx).map_err(|e| crate::HelpError::Storage(e.to_string()))?;

    let mut route_keys: Vec<String> = crate::inventory_route_keys_for_pathname(pathname)
        .into_iter()
        .map(str::to_string)
        .collect();
    // Also query the raw pathname for any legacy rows stored before pattern keys.
    if !route_keys.iter().any(|k| k == pathname) {
        route_keys.push(pathname.to_string());
    }

    let mut out = Vec::new();
    for key in route_keys {
        let rows = HelpTourStepVisit::query(&v)
            .where_user(RecordPredicate::Equals(user.clone()))
            .where_route(StringPredicate::Equals(key))
            .await
            .map_err(|e| crate::HelpError::Storage(e.to_string()))?;
        out.extend(rows.into_iter().map(|row| HelpVisitRecord {
            route: row.route().clone(),
            feature_highlight: row.feature_highlight().clone(),
            spotlight: row.spotlight().cloned(),
            replay: crate::service::replay_from_stored(row.replay()),
        }));
    }

    Ok(out)
}

#[cfg(feature = "ssr")]
async fn mark_steps_seen_ssr(
    steps: &[HelpStepKey],
    local_visits: &[HelpVisitRecord],
) -> Result<(), crate::HelpError> {
    use chrono::Utc;

    let ctx = uf_product::ssr::higgs()
        .await
        .map_err(|e| crate::HelpError::Storage(e.to_string()))?;
    let user_id_str = ctx
        .session_user_id()
        .ok_or(crate::HelpError::Unauthenticated("mark_steps_seen"))?
        .clone();
    let user = uf_product::ssr::session_user_record_id(&user_id_str)
        .map_err(|e| crate::HelpError::Storage(e.to_string()))?;
    let v = uf_product::ssr::valence(&ctx).map_err(|e| crate::HelpError::Storage(e.to_string()))?;
    let now = Utc::now();

    // Merge local-only rows first.
    for local in local_visits {
        upsert_visit(
            &v,
            &user,
            &HelpStepKey {
                route: local.route.clone(),
                feature_highlight: local.feature_highlight.clone(),
                spotlight: local.spotlight.clone(),
            },
            local.replay,
            now,
        )
        .await?;
    }

    for step in steps {
        upsert_visit(&v, &user, step, false, now).await?;
    }
    Ok(())
}

#[cfg(feature = "ssr")]
async fn upsert_visit(
    v: &valence::Valence,
    user: &valence::RecordId,
    step: &HelpStepKey,
    replay: bool,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<(), crate::HelpError> {
    use uf_product::generated::HelpTourStepVisit;
    use valence::{Model, RecordPredicate, StringPredicate};

    let existing = HelpTourStepVisit::query(v)
        .where_user(RecordPredicate::Equals(user.clone()))
        .where_route(StringPredicate::Equals(step.route.clone()))
        .where_feature_highlight(StringPredicate::Equals(step.feature_highlight.clone()))
        .first()
        .await
        .map_err(|e| crate::HelpError::Storage(e.to_string()))?;

    let replay_s = crate::service::replay_to_stored(replay);
    if let Some(row) = existing {
        let mut mutable = row
            .get_mutable(v)
            .set_replay(replay_s)
            .map_err(|e| crate::HelpError::Storage(e.to_string()))?;
        if let Some(spotlight) = &step.spotlight {
            mutable = mutable
                .set_spotlight(spotlight.clone())
                .map_err(|e| crate::HelpError::Storage(e.to_string()))?;
        }
        mutable
            .set_updated_at(now)
            .map_err(|e| crate::HelpError::Storage(e.to_string()))?
            .commit()
            .await
            .map_err(|e| crate::HelpError::Storage(e.to_string()))?;
    } else {
        let new_row = HelpTourStepVisit::new(
            user.clone(),
            step.route.clone(),
            step.feature_highlight.clone(),
            step.spotlight.clone(),
            replay_s,
            now,
            now,
        )
        .map_err(|e| crate::HelpError::Storage(e.to_string()))?;
        HelpTourStepVisit::create(new_row, v)
            .await
            .map_err(|e| crate::HelpError::Storage(e.to_string()))?;
    }
    Ok(())
}

#[cfg(feature = "ssr")]
async fn request_replay_ssr(pathname: &str) -> Result<(), crate::HelpError> {
    use chrono::Utc;
    use uf_product::generated::HelpTourStepVisit;
    use valence::{RecordPredicate, StringPredicate};

    let ctx = uf_product::ssr::higgs()
        .await
        .map_err(|e| crate::HelpError::Storage(e.to_string()))?;
    let user_id_str = ctx
        .session_user_id()
        .ok_or(crate::HelpError::Unauthenticated("request_replay"))?
        .clone();
    let user = uf_product::ssr::session_user_record_id(&user_id_str)
        .map_err(|e| crate::HelpError::Storage(e.to_string()))?;
    let v = uf_product::ssr::valence(&ctx).map_err(|e| crate::HelpError::Storage(e.to_string()))?;
    let now = Utc::now();

    let mut route_keys: Vec<String> = crate::inventory_route_keys_for_pathname(pathname)
        .into_iter()
        .map(str::to_string)
        .collect();
    if !route_keys.iter().any(|k| k == pathname) {
        route_keys.push(pathname.to_string());
    }

    for key in &route_keys {
        let rows = HelpTourStepVisit::query(&v)
            .where_user(RecordPredicate::Equals(user.clone()))
            .where_route(StringPredicate::Equals(key.clone()))
            .await
            .map_err(|e| crate::HelpError::Storage(e.to_string()))?;

        for row in rows {
            row.get_mutable(&v)
                .set_replay(crate::service::replay_to_stored(true))
                .map_err(|e| crate::HelpError::Storage(e.to_string()))?
                .set_updated_at(now)
                .map_err(|e| crate::HelpError::Storage(e.to_string()))?
                .commit()
                .await
                .map_err(|e| crate::HelpError::Storage(e.to_string()))?;
        }
    }

    // Ensure inventory steps without rows also become pending via a replay row.
    // Persist the inventory route key (pattern), not the live slug pathname.
    for step in crate::collect_help_steps_for_route(pathname) {
        upsert_visit(
            &v,
            &user,
            &HelpStepKey {
                route: step.route.to_string(),
                feature_highlight: step.feature_highlight.to_string(),
                spotlight: step.spotlight.map(str::to_string),
            },
            true,
            now,
        )
        .await?;
    }
    Ok(())
}
