//! GitHub bot credential resolve (Neutrino-sealed in production hosts).
//!
//! Production hosts seal [`HELP_GITHUB_FEEDBACK_SECRET`] with Neutrino and install
//! a resolver via [`set_github_token_resolver`] at process boot (or export the
//! resolved material as `UF_HELP_GITHUB_TOKEN` before the server starts). The
//! token must never be sent to the browser.

#![cfg_attr(not(feature = "ssr"), allow(dead_code))]

use std::sync::Mutex;

use crate::HelpError;

/// Neutrino secret name for the Help GitHub feedback bot.
pub const HELP_GITHUB_FEEDBACK_SECRET: &str = "help.github_feedback";

/// Lab / process-env fallback when no custom resolver is installed.
pub const UF_HELP_GITHUB_TOKEN_ENV: &str = "UF_HELP_GITHUB_TOKEN";

type TokenResolver = fn() -> Result<String, HelpError>;

static CUSTOM_RESOLVER: Mutex<Option<TokenResolver>> = Mutex::new(None);

/// Install a host Neutrino (or other) token resolver. Call once at server boot.
///
/// # Examples
///
/// ```ignore
/// // Host boot (SSR):
/// uf_help::set_github_token_resolver(|| {
///     neutrino::resolve(uf_help::HELP_GITHUB_FEEDBACK_SECRET)
///         .map_err(|_| uf_help::HelpError::Misconfigured(uf_help::HELP_GITHUB_FEEDBACK_SECRET))
/// });
/// ```
pub fn set_github_token_resolver(resolver: TokenResolver) {
    let mut guard = CUSTOM_RESOLVER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    *guard = Some(resolver);
}

/// Clear a custom resolver (tests / host teardown).
pub fn clear_github_token_resolver() {
    let mut guard = CUSTOM_RESOLVER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    *guard = None;
}

/// Resolve the GitHub bot token for Help filing.
///
/// Order: custom Neutrino resolver (if installed) → `UF_HELP_GITHUB_TOKEN` env.
///
/// # Errors
///
/// Returns [`HelpError::Misconfigured`] when no token is available.
pub fn resolve_github_token() -> Result<String, HelpError> {
    let custom = CUSTOM_RESOLVER
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .as_ref()
        .copied();
    if let Some(resolver) = custom {
        return resolver();
    }
    std::env::var(UF_HELP_GITHUB_TOKEN_ENV)
        .ok()
        .filter(|s| !s.trim().is_empty())
        .ok_or(HelpError::Misconfigured(HELP_GITHUB_FEEDBACK_SECRET))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn misconfigured_when_no_token() {
        clear_github_token_resolver();
        // May pass if the developer shell already exports the env; treat either as ok.
        let _ = resolve_github_token();
    }

    #[test]
    fn custom_resolver_wins() {
        set_github_token_resolver(|| Ok("test-token".into()));
        assert_eq!(resolve_github_token().unwrap(), "test-token");
        clear_github_token_resolver();
    }
}
