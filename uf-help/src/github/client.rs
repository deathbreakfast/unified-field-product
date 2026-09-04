//! GitHub API client trait and HTTP implementation.

use crate::HelpError;

/// Bug report fields collected from the no-account form.
#[derive(Debug, Clone)]
pub struct BugReportPayload {
    /// Issue title.
    pub title: String,
    /// Description.
    pub description: String,
    /// Steps to reproduce.
    pub steps: String,
    /// Expected behavior.
    pub expected: String,
    /// Actual behavior.
    pub actual: String,
    /// Optional app / version.
    pub app_version: Option<String>,
    /// Optional browser / OS.
    pub browser_os: Option<String>,
    /// Optional contact email.
    pub contact_email: Option<String>,
}

/// Feature request fields.
#[derive(Debug, Clone)]
pub struct FeatureRequestPayload {
    /// Issue title.
    pub title: String,
    /// Problem / use case.
    pub problem: String,
    /// Proposed solution.
    pub proposed: String,
    /// Optional alternatives.
    pub alternatives: Option<String>,
    /// Optional contact email.
    pub contact_email: Option<String>,
}

/// Security report fields (private channel only).
#[derive(Debug, Clone)]
pub struct SecurityReportPayload {
    /// Short summary.
    pub summary: String,
    /// Description and impact.
    pub description: String,
    /// Repro / PoC.
    pub repro: String,
    /// Affected components.
    pub affected: String,
    /// Optional severity.
    pub severity: Option<String>,
    /// Optional contact email.
    pub contact_email: Option<String>,
}

/// Public issue create request.
#[derive(Debug, Clone)]
pub struct CreateIssue {
    /// Issue title.
    pub title: String,
    /// Markdown body.
    pub body: String,
    /// Labels (e.g. `bug`, `enhancement`).
    pub labels: Vec<String>,
}

/// Private vulnerability report payload.
#[derive(Debug, Clone)]
pub struct PrivateVulnReport {
    /// Summary.
    pub summary: String,
    /// Description.
    pub description: String,
    /// Optional severity (`low` / `medium` / `high` / `critical`).
    pub severity: Option<String>,
}

/// Thin GitHub feedback surface used by Help report submit.
#[allow(clippy::double_must_use)]
#[async_trait::async_trait]
pub trait GitHubFeedbackClient: Send + Sync {
    /// Create a public issue. Returns the issue HTML URL.
    ///
    /// # Errors
    ///
    /// Returns [`HelpError::GitHubUpstream`] on transport failure, non-success HTTP
    /// status, or a response missing `html_url`.
    async fn create_issue(
        &self,
        owner: &str,
        repo: &str,
        issue: CreateIssue,
    ) -> Result<String, HelpError>;

    /// File a private vulnerability report (never a public issue).
    ///
    /// # Errors
    ///
    /// Returns [`HelpError::GitHubUpstream`] on transport failure or non-success
    /// HTTP status from the private reporting API.
    async fn create_private_vulnerability_report(
        &self,
        owner: &str,
        repo: &str,
        report: PrivateVulnReport,
    ) -> Result<(), HelpError>;
}

/// Reqwest-backed GitHub client (SSR).
#[cfg(feature = "ssr")]
pub struct HttpGitHubClient {
    token: String,
    http: reqwest::Client,
}

#[cfg(feature = "ssr")]
impl HttpGitHubClient {
    /// Build a client with the given PAT / app token.
    #[must_use]
    pub fn new(token: String) -> Self {
        Self {
            token,
            http: reqwest::Client::new(),
        }
    }
}

#[cfg(feature = "ssr")]
#[async_trait::async_trait]
impl GitHubFeedbackClient for HttpGitHubClient {
    async fn create_issue(
        &self,
        owner: &str,
        repo: &str,
        issue: CreateIssue,
    ) -> Result<String, HelpError> {
        let url = format!("https://api.github.com/repos/{owner}/{repo}/issues");
        let body = serde_json::json!({
            "title": issue.title,
            "body": issue.body,
            "labels": issue.labels,
        });
        let resp = self
            .http
            .post(&url)
            .header("Accept", "application/vnd.github+json")
            .header("Authorization", format!("Bearer {}", self.token))
            .header("User-Agent", "uf-help")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .json(&body)
            .send()
            .await
            .map_err(|e| HelpError::GitHubUpstream(format!("transport: {e}")))?;
        let status = resp.status();
        if !status.is_success() {
            return Err(HelpError::GitHubUpstream(format!(
                "status {}",
                status.as_u16()
            )));
        }
        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| HelpError::GitHubUpstream(format!("json: {e}")))?;
        json.get("html_url")
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .ok_or_else(|| HelpError::GitHubUpstream("missing html_url".into()))
    }

    async fn create_private_vulnerability_report(
        &self,
        owner: &str,
        repo: &str,
        report: PrivateVulnReport,
    ) -> Result<(), HelpError> {
        // Private vulnerability reporting API — never POST /issues for security.
        let url =
            format!("https://api.github.com/repos/{owner}/{repo}/security-advisories/reports");
        let mut body = serde_json::json!({
            "summary": report.summary,
            "description": report.description,
        });
        if let Some(severity) = &report.severity {
            body["severity"] = serde_json::Value::String(severity.clone());
        }
        let resp = self
            .http
            .post(&url)
            .header("Accept", "application/vnd.github+json")
            .header("Authorization", format!("Bearer {}", self.token))
            .header("User-Agent", "uf-help")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .json(&body)
            .send()
            .await
            .map_err(|e| HelpError::GitHubUpstream(format!("transport: {e}")))?;
        let status = resp.status();
        if !status.is_success() {
            return Err(HelpError::GitHubUpstream(format!(
                "status {}",
                status.as_u16()
            )));
        }
        Ok(())
    }
}

#[cfg(not(feature = "ssr"))]
/// Placeholder when SSR is off.
pub struct HttpGitHubClient;

#[cfg(not(feature = "ssr"))]
impl HttpGitHubClient {
    /// Unreachable stub.
    #[must_use]
    pub fn new(_token: String) -> Self {
        Self
    }
}
