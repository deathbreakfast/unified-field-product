use uf_product::preview::PreviewRegistration;

use super::paths::preview_page;

/// Build the preview catalog href for a slug under `/orbital`.
pub fn preview_href(slug: &str) -> String {
    if slug.is_empty() {
        "/orbital".to_string()
    } else {
        preview_page(slug)
    }
}

/// Build the preview catalog href for a registration.
pub fn preview_registration_href(item: &PreviewRegistration) -> String {
    preview_href(item.slug)
}

/// Composite string for AutoComplete filtering (label + slug + section + category).
pub fn preview_registration_search_value(item: &PreviewRegistration) -> String {
    let mut parts = vec![item.label.to_string(), item.slug.to_string()];
    if !item.section.is_empty() {
        parts.push(item.section.to_string());
    }
    if !item.category.is_empty() {
        parts.push(item.category.to_string());
    }
    parts.join(" ")
}

/// Find a registration by its autocomplete search value.
pub fn find_preview_registration_by_search_value<'a>(
    search_value: &str,
    registrations: &[&'a PreviewRegistration],
) -> Option<&'a PreviewRegistration> {
    registrations
        .iter()
        .copied()
        .find(|item| preview_registration_search_value(item) == search_value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use icondata::AiAppstoreOutlined;
    use leptos::prelude::*;

    fn sample_registration(
        slug: &'static str,
        label: &'static str,
        section: &'static str,
        category: &'static str,
    ) -> PreviewRegistration {
        PreviewRegistration {
            slug,
            label,
            section,
            section_priority: 0,
            category,
            category_priority: 0,
            category_default_collapsed: false,
            group: "",
            group_priority: 0,
            nav_item: false,
            icon: AiAppstoreOutlined,
            render: || view! { <span></span> }.into_any(),
        }
    }

    #[test]
    fn preview_href_for_introduction() {
        assert_eq!(preview_href(""), "/orbital");
    }

    #[test]
    fn preview_href_for_component_slug() {
        assert_eq!(preview_href("app-bar"), "/orbital/app-bar");
    }

    #[test]
    fn preview_registration_search_value_includes_metadata() {
        let item = sample_registration("app-bar", "App Bar", "Core Components", "Shell");
        assert_eq!(
            preview_registration_search_value(&item),
            "App Bar app-bar Core Components Shell"
        );
    }

    #[test]
    fn preview_registration_search_value_omits_empty_section_and_category() {
        let item = sample_registration("", "Introduction", "", "");
        assert_eq!(preview_registration_search_value(&item), "Introduction ");
    }
}
