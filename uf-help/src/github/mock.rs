//! Mock GitHub client for tests.

use std::sync::Mutex;

use super::client::{CreateIssue, GitHubFeedbackClient, PrivateVulnReport};
use crate::HelpError;

/// Records calls for assertions; never hits the network.
#[derive(Debug, Default)]
pub struct MockGitHubClient {
    /// Public issues created.
    pub issues: Mutex<Vec<(String, String, CreateIssue)>>,
    /// Private vuln reports created.
    pub vulns: Mutex<Vec<(String, String, PrivateVulnReport)>>,
    /// When true, `create_issue` fails.
    pub fail_issues: bool,
}

#[async_trait::async_trait]
impl GitHubFeedbackClient for MockGitHubClient {
    async fn create_issue(
        &self,
        owner: &str,
        repo: &str,
        issue: CreateIssue,
    ) -> Result<String, HelpError> {
        if self.fail_issues {
            return Err(HelpError::GitHubUpstream("mock fail".into()));
        }
        self.issues
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push((owner.to_string(), repo.to_string(), issue));
        Ok(format!("https://github.com/{owner}/{repo}/issues/1"))
    }

    async fn create_private_vulnerability_report(
        &self,
        owner: &str,
        repo: &str,
        report: PrivateVulnReport,
    ) -> Result<(), HelpError> {
        self.vulns
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push((owner.to_string(), repo.to_string(), report));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_creates_issue_and_forbids_confusion_with_vuln() {
        let client = MockGitHubClient::default();
        let url = client
            .create_issue(
                "acme",
                "widget",
                CreateIssue {
                    title: "t".into(),
                    body: "b".into(),
                    labels: vec!["bug".into()],
                },
            )
            .await
            .unwrap();
        assert!(url.contains("/issues/1"));
        assert_eq!(client.issues.lock().unwrap().len(), 1);
        assert!(client.vulns.lock().unwrap().is_empty());

        client
            .create_private_vulnerability_report(
                "acme",
                "widget",
                PrivateVulnReport {
                    summary: "s".into(),
                    description: "d".into(),
                    severity: Some("high".into()),
                },
            )
            .await
            .unwrap();
        assert_eq!(client.vulns.lock().unwrap().len(), 1);
        // Security path must not create a public issue.
        assert_eq!(client.issues.lock().unwrap().len(), 1);
    }
}
