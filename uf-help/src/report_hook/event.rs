//! Wire types for the report-submitted hook.

use serde::{Deserialize, Serialize};

/// Which report dialog produced a submission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportKind {
    /// Bug report, filed as a labeled GitHub issue.
    Bug,
    /// Feature request, filed as an `enhancement` issue.
    Feature,
    /// Security report, filed as a private vulnerability report.
    Security,
}

/// Published once a report has been filed on GitHub.
///
/// The topic name is `help.report.submitted`, keyed by `app_id`. Contact emails
/// are never part of the event. For [`ReportKind::Security`], `title` is a
/// generic line naming the app, `body_markdown` is `None`, and `reference_url`
/// points at the repository's advisories page, so the report text never reaches
/// the event log.
///
/// Photon records the publisher as a system actor, so subscribers can't recover
/// the submitting user from the event itself. `submitter_user_id` carries the
/// session user's `table:id` record key (`None` for an anonymous submitter) and
/// is copied from the request's session, never from client input.
#[cfg(feature = "photon")]
#[photon::topic(name = "help.report.submitted", keyed_by = "app_id")]
pub struct HelpReportSubmitted {
    /// Which dialog produced the report.
    pub kind: ReportKind,
    /// Route the report was filed from.
    pub route: String,
    /// Registration id of the app that owns the route, such as `valence`.
    pub app_id: String,
    /// Display name of that app.
    pub app_name: String,
    /// GitHub `owner/repo` the report was filed against.
    pub repository: String,
    /// Report title, or a generic line for security reports.
    pub title: String,
    /// Report text without contact details; `None` for security reports.
    pub body_markdown: Option<String>,
    /// Issue URL, or the advisories page for security reports.
    pub reference_url: Option<String>,
    /// Session user's `table:id` record key; `None` when not signed in.
    pub submitter_user_id: Option<String>,
}
