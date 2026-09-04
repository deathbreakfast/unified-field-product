//! Validate and submit Help bug / feature / security reports.

use std::fmt::Write as _;

use crate::github::{
    check_rate_limit, resolve_github_token, BugReportPayload, CreateIssue, FeatureRequestPayload,
    GitHubFeedbackClient, HttpGitHubClient, PrivateVulnReport, SecurityReportPayload,
};
use crate::repository::parse_github_owner_repo;
use crate::HelpError;

const MAX_TITLE: usize = 200;
const MAX_BODY: usize = 8_000;

fn require_nonempty(field: &str, value: &str) -> Result<(), HelpError> {
    if value.trim().is_empty() {
        return Err(HelpError::Validation(format!("{field} is required")));
    }
    Ok(())
}

fn cap(field: &str, value: &str, max: usize) -> Result<(), HelpError> {
    if value.len() > max {
        return Err(HelpError::Validation(format!(
            "{field} exceeds {max} characters"
        )));
    }
    Ok(())
}

fn resolve_owner_repo(route: &str) -> Result<(String, String), HelpError> {
    let repo_url = crate::repository::resolve_help_repository(route)
        .ok_or(HelpError::Misconfigured("app repository"))?;
    parse_github_owner_repo(repo_url).ok_or(HelpError::Misconfigured("app repository url"))
}

/// Submit a labeled bug issue.
pub async fn submit_bug(route: &str, payload: BugReportPayload) -> Result<String, HelpError> {
    require_nonempty("title", &payload.title)?;
    require_nonempty("description", &payload.description)?;
    require_nonempty("steps", &payload.steps)?;
    require_nonempty("expected", &payload.expected)?;
    require_nonempty("actual", &payload.actual)?;
    cap("title", &payload.title, MAX_TITLE)?;
    cap("description", &payload.description, MAX_BODY)?;
    let _guard = check_rate_limit("help-report")?;
    let (owner, repo) = resolve_owner_repo(route)?;
    let token = resolve_github_token()?;
    let client = HttpGitHubClient::new(token);
    let mut body = format!(
        "## Description\n{}\n\n## Steps to reproduce\n{}\n\n## Expected\n{}\n\n## Actual\n{}\n\n## Route\n`{}`\n",
        payload.description, payload.steps, payload.expected, payload.actual, route
    );
    if let Some(v) = &payload.app_version {
        let _ = write!(body, "\n## App / version\n{v}\n");
    }
    if let Some(v) = &payload.browser_os {
        let _ = write!(body, "\n## Browser / OS\n{v}\n");
    }
    if let Some(v) = &payload.contact_email {
        let _ = write!(body, "\n## Contact\n{v}\n");
    }
    client
        .create_issue(
            &owner,
            &repo,
            CreateIssue {
                title: payload.title,
                body,
                labels: vec!["bug".into()],
            },
        )
        .await
}

/// Submit an enhancement issue.
pub async fn submit_feature(
    route: &str,
    payload: FeatureRequestPayload,
) -> Result<String, HelpError> {
    require_nonempty("title", &payload.title)?;
    require_nonempty("problem", &payload.problem)?;
    require_nonempty("proposed", &payload.proposed)?;
    cap("title", &payload.title, MAX_TITLE)?;
    let _guard = check_rate_limit("help-report")?;
    let (owner, repo) = resolve_owner_repo(route)?;
    let token = resolve_github_token()?;
    let client = HttpGitHubClient::new(token);
    let mut body = format!(
        "## Problem / use case\n{}\n\n## Proposed solution\n{}\n\n## Route\n`{}`\n",
        payload.problem, payload.proposed, route
    );
    if let Some(v) = &payload.alternatives {
        let _ = write!(body, "\n## Alternatives\n{v}\n");
    }
    if let Some(v) = &payload.contact_email {
        let _ = write!(body, "\n## Contact\n{v}\n");
    }
    client
        .create_issue(
            &owner,
            &repo,
            CreateIssue {
                title: payload.title,
                body,
                labels: vec!["enhancement".into()],
            },
        )
        .await
}

/// Submit a private vulnerability report (never a public issue).
pub async fn submit_security(route: &str, payload: SecurityReportPayload) -> Result<(), HelpError> {
    require_nonempty("summary", &payload.summary)?;
    require_nonempty("description", &payload.description)?;
    require_nonempty("repro", &payload.repro)?;
    require_nonempty("affected", &payload.affected)?;
    cap("summary", &payload.summary, MAX_TITLE)?;
    let _guard = check_rate_limit("help-report")?;
    let (owner, repo) = resolve_owner_repo(route)?;
    let token = resolve_github_token()?;
    let client = HttpGitHubClient::new(token);
    let description = format!(
        "{}\n\n## Steps / PoC\n{}\n\n## Affected\n{}\n\n## Route\n`{}`\n{}",
        payload.description,
        payload.repro,
        payload.affected,
        route,
        payload
            .contact_email
            .as_ref()
            .map(|e| format!("\n## Contact\n{e}\n"))
            .unwrap_or_default()
    );
    client
        .create_private_vulnerability_report(
            &owner,
            &repo,
            PrivateVulnReport {
                summary: payload.summary,
                description,
                severity: payload.severity,
            },
        )
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::github::MockGitHubClient;

    #[tokio::test]
    async fn security_uses_private_api_not_issues() {
        let client = MockGitHubClient::default();
        client
            .create_private_vulnerability_report(
                "o",
                "r",
                PrivateVulnReport {
                    summary: "s".into(),
                    description: "d".into(),
                    severity: None,
                },
            )
            .await
            .unwrap();
        assert!(client.issues.lock().unwrap().is_empty());
        assert_eq!(client.vulns.lock().unwrap().len(), 1);
    }

    #[test]
    fn validation_rejects_empty_title() {
        let err = require_nonempty("title", "  ").unwrap_err();
        assert!(matches!(err, HelpError::Validation(_)));
    }
}
