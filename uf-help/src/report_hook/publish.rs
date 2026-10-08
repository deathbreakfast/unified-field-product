//! Best-effort publish of `help.report.submitted` after a report is filed.

use super::ReportKind;

/// What `service_reports` hands to the hook once GitHub accepted the report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Submission {
    pub(crate) kind: ReportKind,
    pub(crate) route: String,
    pub(crate) app_id: String,
    pub(crate) app_name: String,
    pub(crate) repository: String,
    pub(crate) title: String,
    pub(crate) body_markdown: Option<String>,
    pub(crate) reference_url: Option<String>,
}

/// App and repository a report was filed against.
pub(crate) struct Target<'a> {
    pub(crate) route: &'a str,
    pub(crate) app_id: &'a str,
    pub(crate) app_name: &'a str,
    pub(crate) owner: &'a str,
    pub(crate) repo: &'a str,
}

impl Submission {
    /// Bug or feature report: title, body without contact details, and issue URL.
    pub(crate) fn issue(
        kind: ReportKind,
        t: &Target<'_>,
        title: &str,
        body: &str,
        url: &str,
    ) -> Self {
        Self {
            kind,
            route: t.route.to_owned(),
            app_id: t.app_id.to_owned(),
            app_name: t.app_name.to_owned(),
            repository: format!("{}/{}", t.owner, t.repo),
            title: title.to_owned(),
            body_markdown: Some(body.to_owned()),
            reference_url: Some(url.to_owned()),
        }
    }

    /// Security report: a generic title and the advisories page, never the report text.
    pub(crate) fn security(t: &Target<'_>) -> Self {
        Self {
            kind: ReportKind::Security,
            route: t.route.to_owned(),
            app_id: t.app_id.to_owned(),
            app_name: t.app_name.to_owned(),
            repository: format!("{}/{}", t.owner, t.repo),
            title: format!("Security report for {}", t.app_name),
            body_markdown: None,
            reference_url: Some(format!(
                "https://github.com/{}/{}/security/advisories",
                t.owner, t.repo
            )),
        }
    }
}

/// Session user's `table:id` key, or `None` for an anonymous request.
#[cfg(feature = "photon")]
async fn session_user_id() -> Option<String> {
    let ctx = uf_product::ssr::higgs().await.ok()?;
    ctx.session_user_id().map(ToString::to_string)
}

/// Publish `help.report.submitted`. Never fails the caller: the report is already filed.
#[allow(clippy::unused_async)] // async for `photon` `.publish().await`; a no-op without it
pub(crate) async fn publish_submitted(submission: Submission) {
    #[cfg(feature = "photon")]
    {
        let kind = match submission.kind {
            ReportKind::Bug => "bug",
            ReportKind::Feature => "feature",
            ReportKind::Security => "security",
        };
        let event = super::HelpReportSubmitted {
            kind: submission.kind,
            route: submission.route,
            app_id: submission.app_id,
            app_name: submission.app_name,
            repository: submission.repository,
            title: submission.title,
            body_markdown: submission.body_markdown,
            reference_url: submission.reference_url,
            submitter_user_id: session_user_id().await,
        };
        if let Err(err) = event.publish().await {
            tracing::warn!(
                target: "uf_help.photon.publish",
                operation = "help.report.publish",
                kind,
                error = %err,
                "help.report.submitted publish failed; report already filed"
            );
        }
    }
    #[cfg(not(feature = "photon"))]
    {
        let _ = submission;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target() -> Target<'static> {
        Target {
            route: "/valence/schemas",
            app_id: "valence",
            app_name: "Valence",
            owner: "unified-field-dev",
            repo: "valence-uf-app",
        }
    }

    #[test]
    fn issue_submission_keeps_text_and_url() {
        let s = Submission::issue(
            ReportKind::Bug,
            &target(),
            "Crash on save",
            "## Description\nboom",
            "https://github.com/unified-field-dev/valence-uf-app/issues/7",
        );
        assert_eq!(s.kind, ReportKind::Bug);
        assert_eq!(s.repository, "unified-field-dev/valence-uf-app");
        assert_eq!(s.title, "Crash on save");
        assert_eq!(s.body_markdown.as_deref(), Some("## Description\nboom"));
        assert!(s.reference_url.unwrap().ends_with("/issues/7"));
    }

    #[test]
    fn security_submission_omits_report_text() {
        let s = Submission::security(&target());
        assert_eq!(s.kind, ReportKind::Security);
        assert_eq!(s.title, "Security report for Valence");
        assert_eq!(s.body_markdown, None);
        assert_eq!(
            s.reference_url.as_deref(),
            Some("https://github.com/unified-field-dev/valence-uf-app/security/advisories")
        );
    }

    #[cfg(feature = "photon")]
    #[tokio::test]
    async fn publish_without_photon_runtime_returns_without_panicking() {
        // No Photon runtime is configured in unit tests: the publish warns and returns.
        publish_submitted(Submission::security(&target())).await;
    }
}
