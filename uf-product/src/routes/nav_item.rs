//! Sidebar entry declared through the `navigation` field of `uf_app!`.

/// One sidebar link declared in `uf_app! { navigation: [...] }`.
///
/// `AppMetadata::navigation()` returns these for every target (not only `ssr`), so
/// `uf_integrations::AppNavLinks` can render them on the server and in the browser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppNavItem {
    /// Link text.
    pub label: &'static str,
    /// Route the link opens, such as `"/magnon/tasks"`.
    pub path: &'static str,
    /// Icon key, such as `"home"`. Unknown keys render no icon.
    pub icon: &'static str,
    /// Hide the link while the session is anonymous.
    pub signed_in_only: bool,
}
