//! Per-hit validators that drop stale index rows at query time.
//!
//! Index rows can outlive the access that produced them: a team you were removed
//! from stays in your index until its writer runs again. A product closes that
//! window by registering a [`WorkspaceSearchHitValidator`] for its
//! `(app_id, source_table)`. [`super::query`] runs every matching validator on
//! each row before it becomes a hit and drops the row when any returns `false`.
//! Rows with no registered validator are kept.

use std::future::Future;
use std::pin::Pin;

use valence::Valence;

/// Future returned by a [`WorkspaceSearchHitValidator`]: `true` keeps the row.
pub type HitValidatorFuture<'a> = Pin<Box<dyn Future<Output = bool> + Send + 'a>>;

/// A query-time check for one `(app_id, source_table)` pair of index rows.
///
/// `validate` receives the caller's session [`Valence`] (never System; [`super::query`]
/// rejects other actors first) and the row's `source_id`. Return `false` to drop the
/// row. Fail closed: return `false` when the lookup itself errors.
///
/// # Examples
///
/// ```rust,ignore
/// use uf_product::workspace_search::{HitValidatorFuture, WorkspaceSearchHitValidator};
///
/// fn team_still_visible<'a>(valence: &'a valence::Valence, source_id: &'a str) -> HitValidatorFuture<'a> {
///     Box::pin(async move { load_team(valence, source_id).await.is_ok_and(|t| t.is_some()) })
/// }
///
/// inventory::submit! {
///     WorkspaceSearchHitValidator::new("polaron", "polaron_team", team_still_visible)
/// }
/// ```
pub struct WorkspaceSearchHitValidator {
    /// App id the rows were written with.
    pub app_id: &'static str,
    /// Source table the rows were written with.
    pub source_table: &'static str,
    /// Returns `true` to keep the row with this `source_id`.
    pub validate: for<'a> fn(&'a Valence, &'a str) -> HitValidatorFuture<'a>,
}

impl WorkspaceSearchHitValidator {
    /// Build a validator for `inventory::submit!`.
    #[must_use]
    pub const fn new(
        app_id: &'static str,
        source_table: &'static str,
        validate: for<'a> fn(&'a Valence, &'a str) -> HitValidatorFuture<'a>,
    ) -> Self {
        Self {
            app_id,
            source_table,
            validate,
        }
    }
}

inventory::collect!(WorkspaceSearchHitValidator);

/// `true` when every validator registered for the row's `(app_id, source_table)` keeps it.
pub(super) async fn keep_hit(
    valence: &Valence,
    app_id: &str,
    source_table: &str,
    source_id: &str,
) -> bool {
    for validator in inventory::iter::<WorkspaceSearchHitValidator> {
        if validator.app_id == app_id
            && validator.source_table == source_table
            && !(validator.validate)(valence, source_id).await
        {
            return false;
        }
    }
    true
}
