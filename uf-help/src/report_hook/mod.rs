//! Let a host react to a Help report once it has been filed.
//!
//! After a bug report, feature request or security report reaches GitHub, Help
//! publishes `help.report.submitted`. A host subscribes to it to do its own work
//! with the report, such as filing a task in its tracker.
//!
//! ## Examples
//!
//! Enable the `photon` feature on `uf-help`, then subscribe in the host. The
//! event doesn't identify the submitter through Photon's actor, so read
//! `submitter_user_id` and skip anonymous reports:
//!
//! ```rust,ignore
//! use uf_help::report_hook::{HelpReportSubmitted, ReportKind};
//!
//! #[photon::subscribe(topic = "help.report.submitted", durable = "my-host.report-tasks")]
//! async fn on_report(
//!     _actor: Box<dyn photon::Actor>,
//!     report: HelpReportSubmitted,
//!     _event: &photon::Event,
//! ) -> photon::Result<()> {
//!     let Some(user) = report.submitter_user_id.as_deref() else {
//!         return Ok(()); // signed-out submitter
//!     };
//!     if report.kind == ReportKind::Security {
//!         assert!(report.body_markdown.is_none()); // security text never leaves GitHub
//!     }
//!     println!("{} filed {:?} on {}", user, report.kind, report.repository);
//!     Ok(())
//! }
//! ```
//!
//! Publishing is best-effort. If Photon isn't configured, Help logs a warning and
//! the report still files. Without the `photon` feature there is nothing to
//! subscribe to.

mod event;
pub(crate) mod publish;

#[cfg(feature = "photon")]
pub use event::HelpReportSubmitted;
pub use event::ReportKind;
