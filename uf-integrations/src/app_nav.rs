//! Sidebar links declared once in `uf_app! { navigation: [...] }`.
//!
//! [`AppNavLinks`] renders one `NavigationLink` per [`AppNavItem`] inside the caller's
//! `NavigationBody`, so a product no longer hand-writes its sidebar.

use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

use leptos::prelude::*;
use uf_product::components::NavigationLink;
use uf_product::routes::AppNavItem;
use uf_product::{use_auth_state, AuthSession};

/// Icon shown for a key that [`icon_for`] does not know.
///
/// `NavigationLink` requires an icon, so an unknown key gets a neutral dot instead.
const FALLBACK_ICON: icondata::Icon = icondata::AiMinusOutlined;

/// Map an `uf_app!` navigation icon key to an `icondata` icon.
///
/// Known keys: `home`, `task-list`, `folder`, `vote`, `target`, `document`, `people`,
/// `settings`. Any other key returns `None`.
#[must_use]
pub fn icon_for(key: &str) -> Option<icondata::Icon> {
    Some(match key {
        "home" => icondata::AiHomeOutlined,
        "task-list" => icondata::AiUnorderedListOutlined,
        "folder" => icondata::AiFolderOutlined,
        "vote" => icondata::AiLikeOutlined,
        "target" => icondata::AiAimOutlined,
        "document" => icondata::AiFileTextOutlined,
        "people" => icondata::AiTeamOutlined,
        "settings" => icondata::AiSettingOutlined,
        _ => return None,
    })
}

/// Log each unknown icon key once per process.
fn warn_unknown_icon(key: &str) {
    static SEEN: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    let seen = SEEN.get_or_init(|| Mutex::new(HashSet::new()));
    let first = seen.lock().is_ok_and(|mut set| set.insert(key.to_owned()));
    if first {
        tracing::warn!(
            icon = key,
            "unknown navigation icon key; using fallback icon"
        );
    }
}

/// Lowercase `label`, with each run of non-alphanumerics collapsed to one `-`.
fn slug(label: &str) -> String {
    label
        .split(|c: char| !c.is_alphanumeric())
        .filter(|part| !part.is_empty())
        .map(str::to_lowercase)
        .collect::<Vec<_>>()
        .join("-")
}

/// Index of the app-root item: the one whose path prefixes every other item's path.
fn root_index(items: &[AppNavItem]) -> Option<usize> {
    if items.len() < 2 {
        return None;
    }
    items.iter().enumerate().find_map(|(i, root)| {
        items
            .iter()
            .enumerate()
            .all(|(j, other)| i == j || other.path.starts_with(root.path))
            .then_some(i)
    })
}

/// Sidebar links from `AppMetadata::navigation()`.
///
/// Render inside the product's `NavigationBody`. Items marked `signed_in` are hidden
/// while the `uf_product::AuthContext` session is anonymous. Selection stays with the
/// product's `NavigationConfig::with_selected_value`.
///
/// ```rust,ignore
/// <NavigationBody slot>
///     <AppNavLinks items=AppMetadata::navigation() />
/// </NavigationBody>
/// ```
#[component]
pub fn AppNavLinks(
    items: &'static [AppNavItem],
    /// Prefix for each link's `data-testid`; the label slug follows it.
    #[prop(default = "nav-")]
    test_id_prefix: &'static str,
) -> impl IntoView {
    let session = use_auth_state();
    let root = root_index(items);

    items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let item = *item;
            let icon = icon_for(item.icon).unwrap_or_else(|| {
                warn_unknown_icon(item.icon);
                FALLBACK_ICON
            });
            let test_id = format!("{test_id_prefix}{}", slug(item.label));
            view! {
                <Show when=move || {
                    !item.signed_in_only
                        || matches!(session.get(), AuthSession::Authenticated(_))
                }>
                    <NavigationLink
                        path=item.path
                        value=item.path
                        icon=icon
                        exact=root == Some(i)
                        test_id=test_id.clone()
                    >
                        {item.label}
                    </NavigationLink>
                </Show>
            }
        })
        .collect_view()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "ssr")]
    use uf_product::components::{
        Navigation, NavigationBody, NavigationConfig, NavigationMaterial,
    };

    const ITEMS: &[AppNavItem] = &[
        AppNavItem {
            label: "Home",
            path: "/magnon",
            icon: "home",
            signed_in_only: false,
        },
        AppNavItem {
            label: "My tasks",
            path: "/magnon/tasks",
            icon: "task-list",
            signed_in_only: true,
        },
    ];

    #[test]
    fn slug_collapses_separators_happy() {
        assert_eq!(slug("My tasks"), "my-tasks");
        assert_eq!(slug("  A & B! "), "a-b");
    }

    #[test]
    fn root_index_finds_prefix_of_all_happy() {
        assert_eq!(root_index(ITEMS), Some(0));
        assert_eq!(root_index(&ITEMS[1..]), None);
    }

    #[test]
    fn icon_for_unknown_key_is_none_sad() {
        assert!(icon_for("home").is_some());
        assert!(icon_for("nope").is_none());
    }

    #[cfg(feature = "ssr")]
    fn render(session: AuthSession) -> String {
        use leptos::reactive::owner::Owner;
        let owner = Owner::new();
        owner.set();
        uf_product::provide_auth_context(session);
        provide_context(leptos_router::location::RequestUrl::new("/magnon"));
        view! {
            <leptos_router::components::Router>
                <Navigation config=NavigationConfig::new()>
                    <NavigationMaterial slot />
                    <NavigationBody slot>
                        <AppNavLinks items=ITEMS />
                    </NavigationBody>
                </Navigation>
            </leptos_router::components::Router>
        }
        .to_html()
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn app_nav_hides_signed_in_items_for_anonymous_sad() {
        let html = render(AuthSession::default());
        assert!(html.contains("nav-home"), "{html}");
        assert!(!html.contains("nav-my-tasks"), "{html}");
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn app_nav_renders_all_items_signed_in_happy() {
        let html = render(AuthSession::Authenticated(uf_product::AuthenticatedUser {
            user_id: "u1".into(),
            email: None,
            display_name: None,
            avatar_url: None,
            roles: vec![],
            email_verified: true,
        }));
        assert!(html.contains("nav-home"), "{html}");
        assert!(html.contains("nav-my-tasks"), "{html}");
    }
}
