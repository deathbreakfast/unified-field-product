//! GitHub feedback client for Help report filing.

#![cfg_attr(not(feature = "ssr"), allow(dead_code, unused_imports))]

mod client;
mod credentials;
mod mock;
mod rate_limit;

pub use client::{
    BugReportPayload, CreateIssue, FeatureRequestPayload, GitHubFeedbackClient, HttpGitHubClient,
    PrivateVulnReport, SecurityReportPayload,
};
pub use credentials::{
    clear_github_token_resolver, resolve_github_token, set_github_token_resolver,
    HELP_GITHUB_FEEDBACK_SECRET, UF_HELP_GITHUB_TOKEN_ENV,
};
pub use mock::MockGitHubClient;
pub use rate_limit::check_rate_limit;
#[allow(unused_imports)] // public API
pub use rate_limit::RateLimitGuard;
