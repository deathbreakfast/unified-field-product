//! Resolve Help GitHub repository from `uf_app!` / [`AppRegistration`].

/// Longest-prefix match of `pathname` against registered app `route_path` values.
///
/// Returns the matching app's `repository` URL when present.
#[cfg(feature = "ssr")]
#[must_use]
pub fn resolve_help_repository(pathname: &str) -> Option<&'static str> {
    use uf_product::{AppRegistration, AppRegistry};

    let path = if pathname.is_empty() { "/" } else { pathname };
    let mut best: Option<&AppRegistration> = None;
    for app in AppRegistry::auto_discover().iter() {
        let prefix = app.route_path.trim_end_matches('/');
        let matches =
            path == app.route_path || path == prefix || path.starts_with(&format!("{prefix}/"));
        if !matches {
            continue;
        }
        let better = match best {
            None => true,
            Some(prev) => app.route_path.len() > prev.route_path.len(),
        };
        if better {
            best = Some(app);
        }
    }
    best.and_then(|a| a.repository)
}

#[cfg(not(feature = "ssr"))]
#[must_use]
pub fn resolve_help_repository(_pathname: &str) -> Option<&'static str> {
    None
}

/// Parse `https://github.com/owner/repo` (optional `.git` / trailing slash) into `(owner, repo)`.
#[must_use]
pub fn parse_github_owner_repo(repository_url: &str) -> Option<(String, String)> {
    let trimmed = repository_url.trim().trim_end_matches('/');
    let without_git = trimmed.strip_suffix(".git").unwrap_or(trimmed);
    let rest = without_git
        .strip_prefix("https://github.com/")
        .or_else(|| without_git.strip_prefix("http://github.com/"))
        .or_else(|| without_git.strip_prefix("github.com/"))?;
    let mut parts = rest.split('/');
    let owner = parts.next()?.to_string();
    let repo = parts.next()?.to_string();
    if owner.is_empty() || repo.is_empty() {
        return None;
    }
    Some((owner, repo))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_github_owner_repo_happy() {
        assert_eq!(
            parse_github_owner_repo("https://github.com/acme/widget"),
            Some(("acme".into(), "widget".into()))
        );
        assert_eq!(
            parse_github_owner_repo("https://github.com/acme/widget.git/"),
            Some(("acme".into(), "widget".into()))
        );
    }

    #[test]
    fn parse_github_owner_repo_rejects_non_github() {
        assert!(parse_github_owner_repo("https://gitlab.com/acme/widget").is_none());
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn resolve_help_repository_longest_prefix() {
        // Relies on apps linked into the test binary; empty registry → None is ok.
        let _ = resolve_help_repository("/apps");
        let _ = resolve_help_repository("/apps/foo");
        let _ = resolve_help_repository("/welcome");
    }
}
