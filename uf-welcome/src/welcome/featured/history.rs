//! Published changes to the featured list: history, versions and restore.
//!
//! Every publish records one [`WelcomeFeaturedChange`] row. The id of the newest
//! row is the published list's version, so a draft started at an older version
//! can be refused instead of silently overwriting someone else's publish.
//!
//! # Errors
//!
//! Fallible entry points return [`FeaturedError`]: [`FeaturedError::NotFound`] when a
//! change id is missing, [`FeaturedError::Service`] for Valence failures, and the
//! catalog errors of [`super::replace_all`] when a publish or restore writes the list.

use chrono::{DateTime, Utc};
use uuid::Uuid;
use valence::{Model, RecordId, Valence};

use crate::generated::WelcomeFeaturedChange;
use crate::welcome::describe::describe_change;

use super::service::{app_name, list, replace_all};
use super::FeaturedError;

/// One recorded change to the featured list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeaturedChangeRow {
    /// Bare Valence record id; the newest row's id is the list's version.
    pub id: String,
    /// User who made the change (`None` for System jobs or a deleted account).
    pub actor: Option<RecordId>,
    /// What changed, in words.
    pub summary: String,
    /// App ids before the change, in order.
    pub before: Vec<String>,
    /// App ids after the change, in order.
    pub after: Vec<String>,
    /// When the change was published.
    pub created_at: DateTime<Utc>,
}

/// Result of [`publish`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublishOutcome {
    /// The list was written and this change recorded.
    Published(FeaturedChangeRow),
    /// The draft matched the published list; nothing was written.
    Unchanged,
    /// Someone published since `based_on_version`; carries their change when known.
    Conflict(Option<FeaturedChangeRow>),
}

fn to_row(row: &WelcomeFeaturedChange) -> FeaturedChangeRow {
    FeaturedChangeRow {
        id: row.id().map(|r| r.id().to_string()).unwrap_or_default(),
        actor: row.actor().cloned(),
        summary: row.summary().clone(),
        before: row.before().clone(),
        after: row.after().clone(),
        created_at: *row.created_at(),
    }
}

/// Record one change row.
///
/// # Errors
///
/// Returns [`FeaturedError::Service`] when the Valence write fails (including a
/// caller without `WelcomeAdmin`).
pub async fn record_change(
    v: &Valence,
    actor: Option<RecordId>,
    summary: String,
    before: Vec<String>,
    after: Vec<String>,
) -> Result<FeaturedChangeRow, FeaturedError> {
    let id = Uuid::new_v4().to_string();
    let row = WelcomeFeaturedChange::new(actor, summary, before, after, Utc::now())
        .map_err(|e| FeaturedError::service("record change", e))?;
    let created = WelcomeFeaturedChange::upsert(&id, row, v, valence::use_!(r"When a curator **publishes the Welcome featured list**, we **save a history entry** with who changed it, what changed in words, and the list before and after, so other curators can see it and undo it from History. Only Welcome admins read or write these entries."))
        .await
        .map_err(|e| FeaturedError::service("record change", e))?;
    let mut out = to_row(&created);
    if out.id.is_empty() {
        out.id = id;
    }
    Ok(out)
}

/// Every recorded change, newest first.
///
/// # Errors
///
/// Returns [`FeaturedError::Service`] when the Valence list query fails.
pub async fn list_changes(v: &Valence) -> Result<Vec<FeaturedChangeRow>, FeaturedError> {
    let rows = WelcomeFeaturedChange::query(v, valence::use_!(r"In **featured history**, we **list the Welcome featured list's past changes** so curators can see who changed it, when, and restore an earlier list. Only Welcome admins can read the history."))
        .order_by_created_at(valence::query::SortDirection::Desc)
        .await
        .map_err(|e| FeaturedError::service("list changes", e))?;
    let mut out: Vec<FeaturedChangeRow> = rows.iter().map(to_row).collect();
    // Mem backends may ignore ORDER BY; keep newest first in-process.
    out.sort_by_key(|row| std::cmp::Reverse(row.created_at));
    Ok(out)
}

/// The newest change, whose id is the published list's version.
///
/// # Errors
///
/// Returns [`FeaturedError::Service`] when the Valence list query fails.
pub async fn latest_change(v: &Valence) -> Result<Option<FeaturedChangeRow>, FeaturedError> {
    Ok(list_changes(v).await?.into_iter().next())
}

/// One change by id.
///
/// # Errors
///
/// Returns [`FeaturedError::Service`] when the Valence read fails.
pub async fn get_change(v: &Valence, id: &str) -> Result<Option<FeaturedChangeRow>, FeaturedError> {
    let row = WelcomeFeaturedChange::get(id, v, valence::use_!(r"In **featured history**, we **load one past change** to the Welcome featured list so a curator can restore the list from before it. Only Welcome admins can read the history."))
        .await
        .map_err(|e| FeaturedError::service("get change", e))?;
    Ok(row.as_ref().map(to_row))
}

/// Delete every change row (e2e seed isolation).
///
/// # Errors
///
/// Returns [`FeaturedError::Service`] when a list or delete fails.
pub async fn clear_history(v: &Valence) -> Result<(), FeaturedError> {
    let table = WelcomeFeaturedChange::table_name();
    let backend = v
        .backend_for_table(table)
        .map_err(|e| FeaturedError::service("clear history", e))?;
    for row in list_changes(v).await? {
        valence::delete_record(backend.as_ref(), table, &row.id, valence::use_!(r"**Test:** the e2e seed **clears the featured history** so each scenario starts with no past changes. CI and developers running the suite only."))
            .await
            .map_err(|e| FeaturedError::service("clear history", e))?;
        valence::read_cache::invalidate(table, &row.id);
    }
    Ok(())
}

fn current_ids(rows: Vec<super::FeaturedAppRow>) -> Vec<String> {
    rows.into_iter().map(|row| row.app_id).collect()
}

/// Publish `app_ids` as the whole featured list if nobody published since `based_on_version`.
///
/// `based_on_version` is the newest change id the draft started from (`None` when
/// the history was empty). The check and the write are separate calls, so two
/// publishes landing in the same instant can both pass; the later one wins.
///
/// # Errors
///
/// Propagates [`replace_all`] failures (unknown or repeated app ids) and
/// [`FeaturedError::Service`] from the history read or write.
pub async fn publish(
    v: &Valence,
    actor: Option<RecordId>,
    app_ids: &[String],
    based_on_version: Option<&str>,
) -> Result<PublishOutcome, FeaturedError> {
    let latest = latest_change(v).await?;
    if latest.as_ref().map(|c| c.id.as_str()) != based_on_version {
        return Ok(PublishOutcome::Conflict(latest));
    }
    let before = current_ids(list(v).await?);
    if before == app_ids {
        return Ok(PublishOutcome::Unchanged);
    }
    replace_all(v, app_ids).await?;
    let summary = describe_change(&before, app_ids, app_name);
    let change = record_change(v, actor, summary, before, app_ids.to_vec()).await?;
    Ok(PublishOutcome::Published(change))
}

/// Put the list back to how it was before change `change_id`, recording that as a new change.
///
/// # Errors
///
/// Returns [`FeaturedError::NotFound`] when `change_id` is unknown, and propagates
/// [`replace_all`] / history failures.
pub async fn restore(
    v: &Valence,
    actor: Option<RecordId>,
    change_id: &str,
) -> Result<FeaturedChangeRow, FeaturedError> {
    let target = get_change(v, change_id)
        .await?
        .ok_or_else(|| FeaturedError::not_found(change_id))?;
    // Drop apps that were unregistered since, so an old list stays publishable.
    let wanted: Vec<String> = target
        .before
        .into_iter()
        .filter(|id| is_registered(id))
        .collect();
    let before = current_ids(list(v).await?);
    replace_all(v, &wanted).await?;
    let summary = format!(
        "Restored an earlier list: {}",
        lower_first(&describe_change(&before, &wanted, app_name))
    );
    record_change(v, actor, summary, before, wanted).await
}

/// Record a change made through the single-row add / remove / reorder calls.
///
/// # Errors
///
/// Propagates [`list`] and [`record_change`] failures.
pub async fn record_since(
    v: &Valence,
    actor: Option<RecordId>,
    before: Vec<String>,
) -> Result<FeaturedChangeRow, FeaturedError> {
    let after = current_ids(list(v).await?);
    let summary = describe_change(&before, &after, app_name);
    record_change(v, actor, summary, before, after).await
}

/// Current featured app ids, in order.
///
/// # Errors
///
/// Propagates [`list`] failures.
pub async fn current_app_ids(v: &Valence) -> Result<Vec<String>, FeaturedError> {
    Ok(current_ids(list(v).await?))
}

fn is_registered(app_id: &str) -> bool {
    uf_product::AppRegistry::auto_discover()
        .iter()
        .any(|reg| reg.id == app_id)
}

fn lower_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_lowercase().chain(chars).collect(),
        None => String::new(),
    }
}
