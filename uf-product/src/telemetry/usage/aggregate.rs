//! Pure aggregation of page-view visit rows into ranked app lists.

use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Spectra event table for main-shell page views.
pub const PAGE_VIEW_LOG_TABLE: &str = "uf_apps_page_view_log";

/// One page-view row used for aggregation (no PII beyond `viewer_key` for filtering).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisitRow {
    /// Registered app id (`AppRegistration.id`).
    pub app_id: String,
    /// Display name from the event (fallback when registry miss).
    pub app_name: String,
    /// Route prefix from the event (fallback when registry miss).
    pub route_prefix: String,
    /// Session user id or `"anonymous"`.
    pub viewer_key: String,
    /// Event timestamp.
    pub ts: DateTime<Utc>,
}

/// Intermediate ranked app before registry resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RankedApp {
    /// App id.
    pub app_id: String,
    /// Fallback label from events.
    pub app_name: String,
    /// Fallback link from events.
    pub route_prefix: String,
    /// Visit count (most-used / popular) or `0` for recent-only.
    pub count: u64,
    /// Latest visit time (recent ranking).
    pub last_ts: DateTime<Utc>,
}

/// Recent apps for a viewer: unique `app_id` ordered by latest `ts` descending.
///
/// Rows whose `viewer_key` does not match `viewer_key` are ignored.
pub fn aggregate_recent(rows: &[VisitRow], viewer_key: &str, limit: usize) -> Vec<RankedApp> {
    if limit == 0 {
        return Vec::new();
    }
    let mut best: HashMap<String, RankedApp> = HashMap::new();
    for row in rows.iter().filter(|r| r.viewer_key == viewer_key) {
        if row.app_id.is_empty() || row.app_id == "shell" {
            continue;
        }
        best.entry(row.app_id.clone())
            .and_modify(|existing| {
                if row.ts > existing.last_ts {
                    existing.last_ts = row.ts;
                    existing.app_name = row.app_name.clone();
                    existing.route_prefix = row.route_prefix.clone();
                }
            })
            .or_insert_with(|| RankedApp {
                app_id: row.app_id.clone(),
                app_name: row.app_name.clone(),
                route_prefix: row.route_prefix.clone(),
                count: 0,
                last_ts: row.ts,
            });
    }
    let mut ranked: Vec<RankedApp> = best.into_values().collect();
    ranked.sort_by(|a, b| {
        b.last_ts
            .cmp(&a.last_ts)
            .then_with(|| a.app_id.cmp(&b.app_id))
    });
    ranked.truncate(limit);
    ranked
}

/// Most-used apps for a viewer: count visits, top-N by count then latest `ts`.
pub fn aggregate_most_used(rows: &[VisitRow], viewer_key: &str, limit: usize) -> Vec<RankedApp> {
    aggregate_counts(rows.iter().filter(|r| r.viewer_key == viewer_key), limit)
}

/// Fleet-wide popular apps: count all viewers, top-N by count then latest `ts`.
pub fn aggregate_popular(rows: &[VisitRow], limit: usize) -> Vec<RankedApp> {
    aggregate_counts(rows.iter(), limit)
}

fn aggregate_counts<'a, I>(rows: I, limit: usize) -> Vec<RankedApp>
where
    I: Iterator<Item = &'a VisitRow>,
{
    if limit == 0 {
        return Vec::new();
    }
    let mut best: HashMap<String, RankedApp> = HashMap::new();
    for row in rows {
        if row.app_id.is_empty() || row.app_id == "shell" {
            continue;
        }
        best.entry(row.app_id.clone())
            .and_modify(|existing| {
                existing.count = existing.count.saturating_add(1);
                if row.ts > existing.last_ts {
                    existing.last_ts = row.ts;
                    existing.app_name = row.app_name.clone();
                    existing.route_prefix = row.route_prefix.clone();
                }
            })
            .or_insert_with(|| RankedApp {
                app_id: row.app_id.clone(),
                app_name: row.app_name.clone(),
                route_prefix: row.route_prefix.clone(),
                count: 1,
                last_ts: row.ts,
            });
    }
    let mut ranked: Vec<RankedApp> = best.into_values().collect();
    ranked.sort_by(|a, b| {
        b.count
            .cmp(&a.count)
            .then_with(|| b.last_ts.cmp(&a.last_ts))
            .then_with(|| a.app_id.cmp(&b.app_id))
    });
    ranked.truncate(limit);
    ranked
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn ts(secs: i64) -> DateTime<Utc> {
        Utc.timestamp_opt(secs, 0).single().expect("ts")
    }

    fn row(app: &str, viewer: &str, secs: i64) -> VisitRow {
        VisitRow {
            app_id: app.into(),
            app_name: app.to_uppercase(),
            route_prefix: format!("/{app}"),
            viewer_key: viewer.into(),
            ts: ts(secs),
        }
    }

    #[test]
    fn aggregate_recent_by_app_dedupes_and_orders() {
        let rows = vec![
            row("counter", "u1", 10),
            row("valence", "u1", 20),
            row("counter", "u1", 30),
            row("chronon", "u1", 25),
            row("valence", "other", 99),
        ];
        let got = aggregate_recent(&rows, "u1", 8);
        assert_eq!(
            got.iter().map(|a| a.app_id.as_str()).collect::<Vec<_>>(),
            vec!["counter", "chronon", "valence"]
        );
        assert_eq!(got[0].last_ts, ts(30));
    }

    #[test]
    fn aggregate_recent_empty() {
        assert!(aggregate_recent(&[], "u1", 8).is_empty());
        assert!(aggregate_recent(&[row("counter", "u1", 1)], "nobody", 8).is_empty());
        assert!(aggregate_recent(&[row("counter", "u1", 1)], "u1", 0).is_empty());
    }

    #[test]
    fn most_used_counts_only_matching_rows() {
        let rows = vec![
            row("counter", "u1", 1),
            row("counter", "u1", 2),
            row("counter", "u2", 3),
            row("valence", "u1", 4),
            row("shell", "u1", 5),
        ];
        let got = aggregate_most_used(&rows, "u1", 8);
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].app_id, "counter");
        assert_eq!(got[0].count, 2);
        assert_eq!(got[1].app_id, "valence");
        assert_eq!(got[1].count, 1);
    }

    #[test]
    fn popular_sorts_by_count_desc() {
        let rows = vec![
            row("a", "u1", 1),
            row("b", "u1", 2),
            row("b", "u2", 3),
            row("b", "u3", 4),
            row("a", "u2", 5),
        ];
        let got = aggregate_popular(&rows, 8);
        assert_eq!(got[0].app_id, "b");
        assert_eq!(got[0].count, 3);
        assert_eq!(got[1].app_id, "a");
        assert_eq!(got[1].count, 2);
    }

    #[test]
    fn popular_respects_limit() {
        let rows = vec![row("a", "u1", 1), row("b", "u1", 2), row("c", "u1", 3)];
        let got = aggregate_popular(&rows, 2);
        assert_eq!(got.len(), 2);
    }
}
