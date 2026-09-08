//! Welcome featured-app catalog CRUD (session Valence + WelcomeAdmin / System).
//!
//! # Errors
//!
//! Fallible entry points return [`FeaturedError`]: [`FeaturedError::UnknownApp`] when
//! `app_id` is missing from [`uf_product::AppRegistry`], [`FeaturedError::Duplicate`] on
//! re-add, [`FeaturedError::NotFound`] on remove/reorder of a missing key, and
//! [`FeaturedError::Service`] for Valence failures. Callers that mutate must pass a
//! [`Valence`] whose actor holds `WelcomeAdmin` (or System for bootstrap / integ).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uf_product::AppRegistry;
use uuid::Uuid;
use valence::{Model, StringPredicate, Valence};

use crate::generated::WelcomeFeaturedApp;

use super::FeaturedError;

/// One featured catalog row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeaturedAppRow {
    /// Bare Valence record id.
    pub id: String,
    /// [`uf_product::AppRegistration::id`].
    pub app_id: String,
    /// Sort key (ascending on list).
    pub ordinal: i64,
    /// When the row was created.
    pub created_at: DateTime<Utc>,
    /// When the row was last updated.
    pub updated_at: DateTime<Utc>,
}

fn record_id_str(row: &WelcomeFeaturedApp) -> String {
    row.id()
        .map(|r| r.id().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_default()
}

fn to_row(row: &WelcomeFeaturedApp) -> FeaturedAppRow {
    FeaturedAppRow {
        id: record_id_str(row),
        app_id: row.app_id().clone(),
        ordinal: *row.ordinal(),
        created_at: *row.created_at(),
        updated_at: *row.updated_at(),
    }
}

fn ensure_known_app(app_id: &str) -> Result<(), FeaturedError> {
    let known = AppRegistry::auto_discover()
        .iter()
        .any(|reg| reg.id == app_id);
    if known {
        Ok(())
    } else {
        Err(FeaturedError::unknown_app(app_id))
    }
}

async fn find_by_app_id(
    v: &Valence,
    app_id: &str,
) -> Result<Option<WelcomeFeaturedApp>, FeaturedError> {
    WelcomeFeaturedApp::query(v)
        .where_app_id(StringPredicate::Equals(app_id.to_string()))
        .first()
        .await
        .map_err(|e| FeaturedError::service("query", e))
}

/// List featured rows ordered by ascending `ordinal`.
///
/// Rows whose `app_id` is no longer in [`AppRegistry`] are omitted (stale catalog).
///
/// # Errors
///
/// Returns [`FeaturedError::Service`] when the Valence list query fails.
pub async fn list(v: &Valence) -> Result<Vec<FeaturedAppRow>, FeaturedError> {
    let rows = WelcomeFeaturedApp::query(v)
        .order_by_ordinal(valence::query::SortDirection::Asc)
        .await
        .map_err(|e| FeaturedError::service("list", e))?;
    let registry = AppRegistry::auto_discover();
    let mut out: Vec<FeaturedAppRow> = rows
        .into_iter()
        .filter(|row| registry.iter().any(|reg| reg.id == row.app_id()))
        .map(|row| to_row(&row))
        .collect();
    // Mem backends may ignore ORDER BY; keep ordinal order in-process.
    out.sort_by_key(|row| row.ordinal);
    Ok(out)
}

/// Remove every featured row (e2e seed isolation).
///
/// # Errors
///
/// Propagates [`list`] / [`remove`] failures (`FeaturedError::Service` or
/// [`FeaturedError::NotFound`] if a row disappears mid-clear).
pub async fn clear_all(v: &Valence) -> Result<(), FeaturedError> {
    let rows = list(v).await?;
    for row in rows {
        remove(v, &row.app_id).await?;
    }
    Ok(())
}

/// Insert a featured row for a registered `app_id` at `ordinal`.
///
/// Expects a System [`Valence`] (create policy is `SYSTEM_ONLY`).
///
/// # Errors
///
/// Returns [`FeaturedError::UnknownApp`] when `app_id` is missing from
/// [`AppRegistry`], [`FeaturedError::Duplicate`] when the app is already featured,
/// or [`FeaturedError::Service`] on Valence create/upsert failure.
pub async fn add(v: &Valence, app_id: &str, ordinal: i64) -> Result<FeaturedAppRow, FeaturedError> {
    ensure_known_app(app_id)?;
    if find_by_app_id(v, app_id).await?.is_some() {
        return Err(FeaturedError::duplicate(app_id));
    }

    let now = Utc::now();
    let id = Uuid::new_v4().to_string();
    let row = WelcomeFeaturedApp::new(app_id.to_string(), ordinal, now, now)
        .map_err(|e| FeaturedError::service("add", e))?;
    let created = WelcomeFeaturedApp::upsert(&id, row, v)
        .await
        .map_err(|e| FeaturedError::service("add", e))?;
    let mut out = to_row(&created);
    if out.id.is_empty() {
        out.id = id;
    }
    Ok(out)
}

/// Remove a featured row by Valence record id **or** `app_id`.
///
/// Expects a System [`Valence`] (delete policy is `SYSTEM_ONLY`).
/// Physically deletes the row (catalog rows do not wait on a Chronon deletion worker).
///
/// # Errors
///
/// Returns [`FeaturedError::NotFound`] when neither record id nor `app_id` matches,
/// or [`FeaturedError::Service`] on Valence get/delete failure (including a row
/// missing a record id).
pub async fn remove(v: &Valence, app_id_or_id: &str) -> Result<(), FeaturedError> {
    let key = app_id_or_id;
    let existing = match WelcomeFeaturedApp::get(key, v)
        .await
        .map_err(|e| FeaturedError::service("remove", e))?
    {
        Some(row) => row,
        None => find_by_app_id(v, key)
            .await?
            .ok_or_else(|| FeaturedError::not_found(key))?,
    };
    let id = {
        let extracted = record_id_str(&existing);
        if extracted.is_empty() {
            return Err(FeaturedError::service(
                "remove",
                anyhow::anyhow!("featured row missing record id for key {key}"),
            ));
        } else {
            extracted
        }
    };
    let _ = existing;
    let backend = v
        .backend_for_table(WelcomeFeaturedApp::table_name())
        .map_err(|e| FeaturedError::service("remove", e))?;
    backend
        .delete_record(WelcomeFeaturedApp::table_name(), &id)
        .await
        .map_err(|e| FeaturedError::service("remove", e))?;
    valence::read_cache::invalidate(WelcomeFeaturedApp::table_name(), &id);
    Ok(())
}

/// Set `ordinal` to `0..n` for the given `app_ids` (full new order).
///
/// Every `app_id` must already be featured; missing keys return [`FeaturedError::NotFound`].
/// Expects a System [`Valence`] (update policy is `SYSTEM_ONLY`).
///
/// # Errors
///
/// Returns [`FeaturedError::NotFound`] when an `app_id` is not featured (or its
/// row has no record id), or [`FeaturedError::Service`] on Valence update failure.
pub async fn reorder(v: &Valence, app_ids: &[String]) -> Result<(), FeaturedError> {
    let now = Utc::now();
    for (ordinal, app_id) in app_ids.iter().enumerate() {
        let row = find_by_app_id(v, app_id)
            .await?
            .ok_or_else(|| FeaturedError::not_found(app_id.clone()))?;
        let id = record_id_str(&row);
        if id.is_empty() {
            return Err(FeaturedError::not_found(app_id.clone()));
        }
        row.get_mutable(v)
            .set_ordinal(ordinal as i64)
            .map_err(|e| FeaturedError::service("reorder", e))?
            .set_updated_at(now)
            .map_err(|e| FeaturedError::service("reorder", e))?
            .commit()
            .await
            .map_err(|e| FeaturedError::service("reorder", e))?;
    }
    Ok(())
}
