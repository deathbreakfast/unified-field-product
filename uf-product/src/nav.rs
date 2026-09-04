//! Navigation helpers from the design system plus product-local history helpers.

pub use orbital_zone_a::nav::*;

use leptos_router::NavigateOptions;

/// Navigate back in the browser history when possible; otherwise go to `fallback`.
///
/// Prefers `history.back()` whenever the session has a prior entry (`length > 1`),
/// including cross-origin referrers (return to wherever the user came from).
/// On SSR or when history is unavailable / empty, calls `navigate(fallback, …)`.
pub fn navigate_back_or(fallback: &str, navigate: &impl Fn(&str, NavigateOptions)) {
    #[cfg(target_arch = "wasm32")]
    {
        use leptos::web_sys;
        if let Some(window) = web_sys::window() {
            if let Ok(history) = window.history() {
                if history.length().unwrap_or(0) > 1 {
                    let _ = history.back();
                    return;
                }
            }
            // Same-tab navigations sometimes report length == 1 while still having
            // a document referrer (e.g. opened from an external link that replaced).
            let referrer = window.document().map(|d| d.referrer()).unwrap_or_default();
            if !referrer.is_empty() {
                if let Ok(history) = window.history() {
                    let _ = history.back();
                    return;
                }
            }
        }
    }
    navigate(fallback, NavigateOptions::default());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[test]
    fn navigate_back_or_uses_fallback_on_ssr_happy_path() {
        let called = RefCell::new(None::<String>);
        navigate_back_or("/apps", &|path, _| {
            *called.borrow_mut() = Some(path.to_string());
        });
        assert_eq!(called.into_inner().as_deref(), Some("/apps"));
    }
}
