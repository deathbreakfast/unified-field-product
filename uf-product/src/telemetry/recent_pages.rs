//! Welcome recent pages: the last page each person opened in each app.
//!
//! Rows live in [`crate::generated::WelcomeRecentPage`], one per user and app,
//! readable and writable only by their owner. Every function runs as the actor
//! on the [`Valence`] it is given and never elevates. Rows expire after
//! [`RECENT_PAGE_TTL_SECS`] (schema TTL) and are removed with the account
//! through the `user` connection.
//!
//! # Concern → API
//!
//! | Concern | API |
//! |---------|-----|
//! | Save the page just opened | [`record_recent_page`] |
//! | Last page per app, newest first | [`list_recent_pages`] |
//! | Clear the caller's history | [`clear_recent_pages`] |

use std::fmt;

use chrono::{DateTime, Duration, Utc};
use valence::{Model, RecordId, RecordPredicate, Valence};

use crate::generated::WelcomeRecentPage;

/// Schema TTL window (30 days), matching `ttl.seconds` on the Valence schema.
pub const RECENT_PAGE_TTL_SECS: u64 = 2_592_000;

/// Most recent pages Welcome shows.
pub const RECENT_PAGES_LIMIT: usize = 8;

const TABLE: &str = "welcome_recent_page";

/// One saved recent page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecentPageRow {
    /// Registered app id (`AppRegistration.id`).
    pub app_id: String,
    /// Page title (last breadcrumb item, or the app name).
    pub title: String,
    /// Pathname that was opened.
    pub route: String,
    /// When the page was opened.
    pub viewed_at: DateTime<Utc>,
}

/// Failure reading or writing recent pages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecentPagesError {
    /// Operation that failed (`record`, `list`, `clear`).
    pub operation: &'static str,
    /// Backend or validation message (no viewer ids or paths).
    pub message: String,
}

impl RecentPagesError {
    fn new(operation: &'static str, message: impl fmt::Display) -> Self {
        Self {
            operation,
            message: message.to_string(),
        }
    }
}

impl fmt::Display for RecentPagesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "recent pages {} failed: {}",
            self.operation, self.message
        )
    }
}

impl std::error::Error for RecentPagesError {}

/// Deterministic row id for `(user, app_id)`.
#[must_use]
pub fn recent_page_id(user: &RecordId, app_id: &str) -> String {
    format!(
        "{}__{}",
        sanitize_id_part(user.id()),
        sanitize_id_part(app_id)
    )
}

fn sanitize_id_part(part: &str) -> String {
    part.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => c,
            _ => '_',
        })
        .collect()
}

/// Save `route` as the last page `user` opened in `app_id`.
///
/// Replaces the user's row for that app. The old row is removed first so the
/// schema TTL restarts on create (sliding 30-day retention). The shell
/// pseudo-app, empty app ids, and routes that are not a same-site path
/// (see [`is_site_path`]) are skipped.
///
/// # Errors
///
/// Returns [`RecentPagesError`] when the actor is not `user`, or when the
/// Valence delete or create fails.
pub async fn record_recent_page(
    valence: &Valence,
    user: &RecordId,
    app_id: &str,
    title: &str,
    route: &str,
    viewed_at: DateTime<Utc>,
) -> Result<(), RecentPagesError> {
    if app_id.is_empty() || app_id == "shell" || !is_site_path(route) {
        return Ok(());
    }
    let id = recent_page_id(user, app_id);
    hard_remove(valence, &id, user, "record").await?;
    let row = WelcomeRecentPage::new(
        user.clone(),
        app_id.to_string(),
        title.to_string(),
        route.to_string(),
        viewed_at,
    )
    .map_err(|e| RecentPagesError::new("record", e))?;
    WelcomeRecentPage::upsert(&id, row, valence, valence::use_!(r"When you open a page in an app, we **save that page's title and address as the last page you opened in that app**, replacing the one saved before. Welcome lists it under **Pick up where you left off** so you can go straight back. Only you can read it, and it is deleted after **30 days**, when you clear your history, or when your account is deleted."))
        .await
        .map_err(|e| RecentPagesError::new("record", e))?;
    Ok(())
}

/// Whether `route` is a path on this site: it starts with one `/`. The route
/// comes from the client and Welcome renders it as a link, so `javascript:`,
/// absolute URLs and protocol-relative `//host` values are never saved.
#[must_use]
pub fn is_site_path(route: &str) -> bool {
    route.starts_with('/') && !route.starts_with("//") && !route.starts_with("/\\")
}

/// Last page `user` opened in each app, newest first, at most `limit`, from
/// the last `lookback`.
///
/// The lookback filter also covers backends that do not expire rows on TTL.
///
/// # Errors
///
/// Returns [`RecentPagesError`] when the Valence query fails.
pub async fn list_recent_pages(
    valence: &Valence,
    user: &RecordId,
    limit: usize,
    lookback: Duration,
) -> Result<Vec<RecentPageRow>, RecentPagesError> {
    let rows = WelcomeRecentPage::query(valence, valence::use_!(r"On **Welcome**, we **load the last page you opened in each app** (its title, address and when you opened it) and **show them to you** under **Pick up where you left off**. Only your own pages are read; nobody else can see them."))
        .where_user(RecordPredicate::Equals(user.clone()))
        .await
        .map_err(|e| RecentPagesError::new("list", e))?;
    Ok(newest_within(
        rows.iter().map(|row| RecentPageRow {
            app_id: row.app_id().clone(),
            title: row.title().clone(),
            route: row.route().clone(),
            viewed_at: *row.viewed_at(),
        }),
        Utc::now() - lookback,
        limit,
    ))
}

/// Keep rows at or after `since`, newest first, at most `limit`.
fn newest_within(
    rows: impl Iterator<Item = RecentPageRow>,
    since: DateTime<Utc>,
    limit: usize,
) -> Vec<RecentPageRow> {
    let mut out: Vec<RecentPageRow> = rows.filter(|row| row.viewed_at >= since).collect();
    out.sort_by(|a, b| {
        b.viewed_at
            .cmp(&a.viewed_at)
            .then_with(|| a.app_id.cmp(&b.app_id))
    });
    out.truncate(limit);
    out
}

/// Delete every recent page `user` has, of any age. Returns how many rows went.
///
/// # Errors
///
/// Returns [`RecentPagesError`] when the actor is not `user`, or when the
/// Valence query or delete fails.
pub async fn clear_recent_pages(
    valence: &Valence,
    user: &RecordId,
) -> Result<usize, RecentPagesError> {
    let rows = WelcomeRecentPage::query(valence, valence::use_!(r"When you **clear your history** on Welcome, we **find every page we saved as the last one you opened in each app** so each can be deleted. Only your own pages are read."))
        .where_user(RecordPredicate::Equals(user.clone()))
        .await
        .map_err(|e| RecentPagesError::new("clear", e))?;
    let mut removed = 0;
    for row in rows {
        hard_remove(valence, &recent_page_id(user, row.app_id()), user, "clear").await?;
        removed += 1;
    }
    Ok(removed)
}

/// Physically delete one row after checking that the actor and the row both
/// belong to `expected_user`.
///
/// Uses a backend delete rather than [`Model::delete`], which only marks
/// `pending_deletion` and would block the re-create in [`record_recent_page`]
/// until a deletion worker runs (same reason as the workspace search writer).
async fn hard_remove(
    valence: &Valence,
    id: &str,
    expected_user: &RecordId,
    operation: &'static str,
) -> Result<(), RecentPagesError> {
    let Some(actor_uid) = valence.actor().user_id() else {
        return Err(RecentPagesError::new(
            operation,
            "only the signed-in owner can change recent pages",
        ));
    };
    if bare_user_id(actor_uid) != expected_user.id() {
        return Err(RecentPagesError::new(
            operation,
            "actor is not the recent page owner",
        ));
    }

    let backend = valence
        .backend_for_table(TABLE)
        .map_err(|e| RecentPagesError::new(operation, e))?;

    // The backend delete skips Valence privacy, so check the stored owner first.
    match valence::get_record(backend.as_ref(), TABLE, id, valence::use_!(r"Before we **replace or clear a page saved as the last one you opened in an app**, we **load that saved page to check it is yours**. The result is only used for that check.")).await {
        Ok(Some(existing)) => {
            let row_user = existing
                .get("user")
                .and_then(|u| u.get("id").and_then(|v| v.as_str()).or_else(|| u.as_str()))
                .unwrap_or("");
            if bare_user_id(row_user) != expected_user.id() {
                return Err(RecentPagesError::new(operation, "recent page owner mismatch"));
            }
        }
        Ok(None) | Err(valence::Error::NotFound(_)) => return Ok(()),
        Err(e) => return Err(RecentPagesError::new(operation, e)),
    }

    match valence::delete_record(backend.as_ref(), TABLE, id, valence::use_!(r"When you open a new page in an app or **clear your history**, we **delete the page saved as the last one you opened in that app**, so it no longer shows on Welcome.")).await {
        Ok(()) | Err(valence::Error::NotFound(_)) => {}
        Err(e) => return Err(RecentPagesError::new(operation, e)),
    }
    valence::read_cache::invalidate(TABLE, id);
    Ok(())
}

fn bare_user_id(id: &str) -> &str {
    id.strip_prefix("user:").unwrap_or(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use chrono::TimeZone;
    use valence::{Actor, InMemoryBackend, SchemaRegistry};

    fn mem_valence(user: &str) -> Valence {
        let _ = SchemaRegistry::global();
        valence::deletion::register_noop_deletion_dispatcher_for_tests();
        Valence::builder()
            .add_backend("default", Arc::new(InMemoryBackend::new()))
            .with_actor(as_user(user))
            .build()
            .expect("in-memory valence")
    }

    fn as_user(user: &str) -> Actor {
        Actor::User {
            user_id: user.into(),
        }
    }

    fn user(id: &str) -> RecordId {
        RecordId::new("user", id)
    }

    async fn record(v: &Valence, owner: &str, app: &str, route: &str, ago: Duration) {
        record_recent_page(v, &user(owner), app, route, route, Utc::now() - ago)
            .await
            .expect("record");
    }

    async fn list(v: &Valence, owner: &str) -> Vec<RecentPageRow> {
        list_recent_pages(v, &user(owner), RECENT_PAGES_LIMIT, Duration::days(30))
            .await
            .expect("list")
    }

    #[test]
    fn schema_is_owner_only_with_30_day_ttl() {
        let meta = SchemaRegistry::global()
            .get_schema(TABLE)
            .expect("schema registered");
        let policies = meta.schema.policies.as_ref().expect("policies");
        let read = policies.read.as_ref().expect("read policies");
        assert!(read.always_allow.is_empty());
        assert!(!read.allow.is_empty());
        assert!(
            !read
                .allow
                .iter()
                .any(|r| r.name.eq_ignore_ascii_case("authenticated")),
            "recent pages must not be readable by every signed-in user"
        );
        assert_eq!(RECENT_PAGE_TTL_SECS, 30 * 24 * 60 * 60);
    }

    #[tokio::test]
    async fn keeps_only_last_page_per_app_newest_first() {
        let v = mem_valence("alice");
        record(&v, "alice", "magnon", "/magnon/1", Duration::minutes(30)).await;
        record(&v, "alice", "valence", "/valence", Duration::minutes(20)).await;
        record(&v, "alice", "magnon", "/magnon/2", Duration::minutes(10)).await;

        let got = list(&v, "alice").await;
        assert_eq!(
            got.iter().map(|r| r.route.as_str()).collect::<Vec<_>>(),
            vec!["/magnon/2", "/valence"]
        );
    }

    #[tokio::test]
    async fn skips_shell_and_old_pages_sad() {
        let v = mem_valence("alice");
        record(&v, "alice", "shell", "/", Duration::minutes(1)).await;
        record(&v, "alice", "magnon", "/magnon", Duration::days(31)).await;
        record(
            &v,
            "alice",
            "notes",
            "javascript:alert(1)",
            Duration::minutes(1),
        )
        .await;
        record(&v, "alice", "docs", "//evil.example", Duration::minutes(1)).await;
        assert!(list(&v, "alice").await.is_empty());
    }

    #[test]
    fn is_site_path_accepts_only_same_site_paths() {
        assert!(is_site_path("/magnon/tasks/42"));
        assert!(is_site_path("/"));
        assert!(!is_site_path("javascript:alert(1)"));
        assert!(!is_site_path("https://evil.example"));
        assert!(!is_site_path("//evil.example"));
        assert!(!is_site_path("/\\evil.example"));
        assert!(!is_site_path(""));
    }

    #[tokio::test]
    async fn caps_at_limit() {
        let v = mem_valence("alice");
        for i in 0..10 {
            let app = format!("app{i}");
            record(&v, "alice", &app, &format!("/{app}"), Duration::minutes(i)).await;
        }
        let got = list(&v, "alice").await;
        assert_eq!(got.len(), RECENT_PAGES_LIMIT);
        assert_eq!(got[0].app_id, "app0");
    }

    #[tokio::test]
    async fn other_user_cannot_read_or_clear_sad() {
        let alice = mem_valence("alice");
        record(&alice, "alice", "magnon", "/magnon", Duration::minutes(1)).await;

        let bob = alice.with_actor(as_user("bob"));
        let peek = list_recent_pages(&bob, &user("alice"), 8, Duration::days(30)).await;
        assert!(
            peek.map_or(true, |rows| rows.is_empty()),
            "bob must not see alice's pages"
        );
        let _ = clear_recent_pages(&bob, &user("alice")).await;
        assert_eq!(
            list(&alice, "alice").await.len(),
            1,
            "bob must not clear alice's pages"
        );
    }

    #[tokio::test]
    async fn clear_empties_recent_pages() {
        let v = mem_valence("alice");
        record(&v, "alice", "magnon", "/magnon", Duration::minutes(2)).await;
        record(&v, "alice", "valence", "/valence", Duration::days(40)).await;

        assert_eq!(
            clear_recent_pages(&v, &user("alice")).await.expect("clear"),
            2
        );
        assert!(list(&v, "alice").await.is_empty());
        assert!(
            list_recent_pages(&v, &user("alice"), 8, Duration::days(365))
                .await
                .expect("list")
                .is_empty()
        );
    }

    #[tokio::test]
    async fn record_for_another_user_is_refused_sad() {
        let v = mem_valence("bob");
        let err = record_recent_page(&v, &user("alice"), "magnon", "/m", "/m", Utc::now())
            .await
            .expect_err("bob cannot write alice's page");
        assert_eq!(err.operation, "record");
    }

    fn row(app: &str, secs: i64) -> RecentPageRow {
        RecentPageRow {
            app_id: app.into(),
            title: app.to_uppercase(),
            route: format!("/{app}"),
            viewed_at: Utc.timestamp_opt(secs, 0).single().expect("ts"),
        }
    }

    #[test]
    fn recent_page_id_is_per_user_and_app() {
        let user = RecordId::new("user", "alice");
        assert_eq!(recent_page_id(&user, "counter"), "alice__counter");
        assert_eq!(recent_page_id(&user, "a/b c"), "alice__a_b_c");
    }

    #[test]
    fn newest_within_orders_and_limits() {
        let since = Utc.timestamp_opt(5, 0).single().expect("ts");
        let got = newest_within(
            vec![row("a", 10), row("old", 1), row("b", 30), row("c", 20)].into_iter(),
            since,
            2,
        );
        assert_eq!(
            got.iter().map(|r| r.app_id.as_str()).collect::<Vec<_>>(),
            vec!["b", "c"]
        );
    }

    #[test]
    fn newest_within_drops_rows_before_window_sad() {
        let since = Utc.timestamp_opt(100, 0).single().expect("ts");
        assert!(newest_within(vec![row("a", 10)].into_iter(), since, 8).is_empty());
    }
}
