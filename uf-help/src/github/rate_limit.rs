//! In-memory rate limit for Help report submits.

#![cfg_attr(not(feature = "ssr"), allow(dead_code))]

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::HelpError;

static RATE_LIMIT: Mutex<Option<HashMap<String, Vec<Instant>>>> = Mutex::new(None);

const WINDOW: Duration = Duration::from_secs(60);
const MAX_PER_WINDOW: usize = 5;

/// Opaque guard returned after a successful rate-limit check.
#[derive(Debug, Default)]
pub struct RateLimitGuard;

/// Check (and record) a submit for `bucket` (e.g. client IP hash or `"anon"`).
///
/// # Errors
///
/// Returns [`HelpError::RateLimited`] when the bucket exceeds the window budget.
pub fn check_rate_limit(bucket: &str) -> Result<RateLimitGuard, HelpError> {
    let mut guard = RATE_LIMIT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let map = guard.get_or_insert_with(HashMap::new);
    let now = Instant::now();
    let entries = map.entry(bucket.to_string()).or_default();
    entries.retain(|t| now.duration_since(*t) < WINDOW);
    if entries.len() >= MAX_PER_WINDOW {
        return Err(HelpError::RateLimited {
            retry_after_secs: WINDOW.as_secs(),
        });
    }
    entries.push(now);
    Ok(RateLimitGuard)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_limit_allows_then_blocks() {
        let key = format!("test-{}", uuid::Uuid::new_v4());
        for _ in 0..MAX_PER_WINDOW {
            assert!(check_rate_limit(&key).is_ok());
        }
        assert!(matches!(
            check_rate_limit(&key),
            Err(HelpError::RateLimited { .. })
        ));
    }
}
