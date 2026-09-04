//! Spectra query + AppRegistry resolution for usage shortcuts.

use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use spectra::Spectra;
use spectra_core::{
    EventRow, EventsQueryFilter, GridFilterItem, GridFilterModel, GridFilterOperator,
};

use super::aggregate::{
    aggregate_most_used, aggregate_popular, aggregate_recent, RankedApp, VisitRow,
    PAGE_VIEW_LOG_TABLE,
};
use super::error::UsageQueryError;
use crate::routes::AppRegistry;

/// Tunables for welcome usage queries.
#[derive(Debug, Clone)]
pub struct UsageQueryOptions {
    /// Max apps returned per list.
    pub limit_apps: usize,
    /// Max event rows to pull from Spectra before aggregating.
    pub lookback_events: u32,
    /// How far back to query (wall clock).
    pub lookback: Duration,
}

impl Default for UsageQueryOptions {
    fn default() -> Self {
        Self {
            limit_apps: 8,
            lookback_events: 500,
            lookback: Duration::days(30),
        }
    }
}

/// App shortcut suitable for welcome cards (no viewer PII).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsageAppLink {
    /// Registered app id.
    pub app_id: String,
    /// Display label.
    pub label: String,
    /// Navigation href.
    pub link: String,
}

/// Recent apps for `viewer_key` (session user id).
///
/// # Errors
///
/// Returns [`UsageQueryError`] from [`recent_apps_for_viewer_on`] when Spectra
/// query fails ([`UsageQueryError::QueryFailed`]).
pub async fn recent_apps_for_viewer(
    spectra: &Spectra,
    viewer_key: &str,
    opts: &UsageQueryOptions,
) -> Result<Vec<UsageAppLink>, UsageQueryError> {
    recent_apps_for_viewer_on(spectra.router().as_ref(), viewer_key, opts).await
}

/// Recent apps against an explicit Spectra router (e2e hosts / tests).
///
/// # Errors
///
/// Returns [`UsageQueryError::QueryFailed`] when the Spectra events query fails.
pub async fn recent_apps_for_viewer_on(
    router: &spectra_core::SpectraRouter,
    viewer_key: &str,
    opts: &UsageQueryOptions,
) -> Result<Vec<UsageAppLink>, UsageQueryError> {
    let rows = fetch_visits(router, Some(viewer_key), opts).await?;
    Ok(resolve_usage_links(&aggregate_recent(
        &rows,
        viewer_key,
        opts.limit_apps,
    )))
}

/// Most-used apps for `viewer_key`.
///
/// # Errors
///
/// Returns [`UsageQueryError`] from [`most_used_for_viewer_on`] when Spectra
/// query fails ([`UsageQueryError::QueryFailed`]).
pub async fn most_used_for_viewer(
    spectra: &Spectra,
    viewer_key: &str,
    opts: &UsageQueryOptions,
) -> Result<Vec<UsageAppLink>, UsageQueryError> {
    most_used_for_viewer_on(spectra.router().as_ref(), viewer_key, opts).await
}

/// Most-used apps against an explicit Spectra router.
///
/// # Errors
///
/// Returns [`UsageQueryError::QueryFailed`] when the Spectra events query fails.
pub async fn most_used_for_viewer_on(
    router: &spectra_core::SpectraRouter,
    viewer_key: &str,
    opts: &UsageQueryOptions,
) -> Result<Vec<UsageAppLink>, UsageQueryError> {
    let rows = fetch_visits(router, Some(viewer_key), opts).await?;
    Ok(resolve_usage_links(&aggregate_most_used(
        &rows,
        viewer_key,
        opts.limit_apps,
    )))
}

/// Fleet-wide popular apps (authenticated callers only — enforced by welcome).
///
/// # Errors
///
/// Returns [`UsageQueryError`] from [`popular_apps_on`] when Spectra query fails
/// ([`UsageQueryError::QueryFailed`]).
pub async fn popular_apps(
    spectra: &Spectra,
    opts: &UsageQueryOptions,
) -> Result<Vec<UsageAppLink>, UsageQueryError> {
    popular_apps_on(spectra.router().as_ref(), opts).await
}

/// Popular apps against an explicit Spectra router.
///
/// # Errors
///
/// Returns [`UsageQueryError::QueryFailed`] when the Spectra events query fails.
pub async fn popular_apps_on(
    router: &spectra_core::SpectraRouter,
    opts: &UsageQueryOptions,
) -> Result<Vec<UsageAppLink>, UsageQueryError> {
    let rows = fetch_visits(router, None, opts).await?;
    Ok(resolve_usage_links(&aggregate_popular(
        &rows,
        opts.limit_apps,
    )))
}

/// Map ranked apps through [`AppRegistry`], falling back to event fields.
pub fn resolve_usage_links(ranked: &[RankedApp]) -> Vec<UsageAppLink> {
    let registry = AppRegistry::auto_discover();
    ranked
        .iter()
        .filter_map(|app| {
            if let Some(reg) = registry.iter().find(|r| r.id == app.app_id) {
                return Some(UsageAppLink {
                    app_id: app.app_id.clone(),
                    label: reg.name.to_string(),
                    link: reg.route_path.to_string(),
                });
            }
            if app.route_prefix.is_empty() {
                return None;
            }
            let label = if app.app_name.is_empty() {
                app.app_id.clone()
            } else {
                app.app_name.clone()
            };
            Some(UsageAppLink {
                app_id: app.app_id.clone(),
                label,
                link: app.route_prefix.clone(),
            })
        })
        .collect()
}

async fn fetch_visits(
    router: &spectra_core::SpectraRouter,
    viewer_key: Option<&str>,
    opts: &UsageQueryOptions,
) -> Result<Vec<VisitRow>, UsageQueryError> {
    let now = Utc::now();
    let mut filter = GridFilterModel::default();
    if let Some(viewer) = viewer_key {
        filter.items.push(GridFilterItem {
            field: "viewer_key".into(),
            operator: GridFilterOperator::Equals,
            value: Value::String(viewer.to_string()),
        });
    }

    let rows = router
        .query_events(EventsQueryFilter {
            table: PAGE_VIEW_LOG_TABLE.into(),
            start: Some(now - opts.lookback),
            end: Some(now + Duration::seconds(5)),
            limit: Some(opts.lookback_events),
            sort_field: Some("ts".into()),
            sort_desc: true,
            filter,
            ..Default::default()
        })
        .await
        .map_err(|e| UsageQueryError::QueryFailed {
            cause: e.to_string(),
        })?;

    Ok(rows.into_iter().filter_map(visit_from_event_row).collect())
}

/// Parse a Spectra event row into a [`VisitRow`].
pub fn visit_from_event_row(row: EventRow) -> Option<VisitRow> {
    let fields = row.fields.as_object()?;
    let app_id = fields.get("app_id")?.as_str()?.to_string();
    let app_name = fields
        .get("app_name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let route_prefix = fields
        .get("route_prefix")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let viewer_key = fields
        .get("viewer_key")
        .and_then(|v| v.as_str())
        .unwrap_or("anonymous")
        .to_string();
    Some(VisitRow {
        app_id,
        app_name,
        route_prefix,
        viewer_key,
        ts: row.ts,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use serde_json::json;

    #[test]
    fn visit_from_event_row_parses_fields() {
        let row = EventRow {
            ts: Utc.timestamp_opt(100, 0).single().expect("ts"),
            fields: json!({
                "app_id": "counter",
                "app_name": "Counter",
                "route_prefix": "/counter",
                "viewer_key": "user-1",
            }),
        };
        let visit = visit_from_event_row(row).expect("visit");
        assert_eq!(visit.app_id, "counter");
        assert_eq!(visit.viewer_key, "user-1");
        assert_eq!(visit.route_prefix, "/counter");
    }

    #[test]
    fn visit_from_event_row_rejects_missing_app_id() {
        let row = EventRow {
            ts: Utc.timestamp_opt(100, 0).single().expect("ts"),
            fields: json!({ "viewer_key": "user-1" }),
        };
        assert!(visit_from_event_row(row).is_none());
    }

    #[test]
    fn resolve_usage_links_falls_back_to_event_fields() {
        let ranked = vec![RankedApp {
            app_id: "not-registered-xyz".into(),
            app_name: "Fallback Name".into(),
            route_prefix: "/fallback".into(),
            count: 3,
            last_ts: Utc.timestamp_opt(1, 0).single().expect("ts"),
        }];
        let links = resolve_usage_links(&ranked);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].label, "Fallback Name");
        assert_eq!(links[0].link, "/fallback");
    }

    #[test]
    fn resolve_usage_links_skips_unknown_without_route() {
        let ranked = vec![RankedApp {
            app_id: "ghost".into(),
            app_name: String::new(),
            route_prefix: String::new(),
            count: 1,
            last_ts: Utc.timestamp_opt(1, 0).single().expect("ts"),
        }];
        assert!(resolve_usage_links(&ranked).is_empty());
    }
}
