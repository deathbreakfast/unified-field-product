//! Step 1 deep-link helpers for Help report dialogs.

use super::HelpReportKind;

/// Build a GitHub deep link for the report kind from an app `repository` URL.
#[must_use]
pub fn github_deep_link(kind: HelpReportKind, repository: &str) -> Option<String> {
    let base = repository.trim().trim_end_matches('/');
    if base.is_empty() {
        return None;
    }
    Some(match kind {
        HelpReportKind::Bug => format!("{base}/issues/new?labels=bug"),
        HelpReportKind::Feature => format!("{base}/issues/new?labels=enhancement"),
        HelpReportKind::Security => format!("{base}/security/advisories/new"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bug_and_feature_use_issues_with_labels() {
        let repo = "https://github.com/acme/widget";
        assert_eq!(
            github_deep_link(HelpReportKind::Bug, repo).as_deref(),
            Some("https://github.com/acme/widget/issues/new?labels=bug")
        );
        assert_eq!(
            github_deep_link(HelpReportKind::Feature, repo).as_deref(),
            Some("https://github.com/acme/widget/issues/new?labels=enhancement")
        );
    }

    #[test]
    fn security_uses_private_advisories_never_issues() {
        let href = github_deep_link(HelpReportKind::Security, "https://github.com/acme/widget/")
            .expect("link");
        assert!(href.ends_with("/security/advisories/new"));
        assert!(!href.contains("/issues"));
    }

    #[test]
    fn empty_repository_yields_none() {
        assert!(github_deep_link(HelpReportKind::Bug, "   ").is_none());
    }
}
