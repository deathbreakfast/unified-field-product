use leptos::prelude::*;
use uf_product::components::{
    NavigationCategory, NavigationCategoryHeader, NavigationLink, NavigationSubItemGroup,
};
use uf_product::preview::PreviewRegistration;

use super::{collect_preview_registrations, preview_page};

type CategoryGroup = (String, Vec<&'static PreviewRegistration>);

/// Author-first preview nav order (18 categories).
const CATEGORY_ORDER: &[&str] = &[
    "Layout",
    "Typography",
    "Theme",
    "Surfaces",
    "Inputs",
    "Calendar & Time",
    "Data Display",
    "Feedback",
    "Navigation",
    "Shell",
    "Site",
    "Sections",
    "Motion",
    "Data Table",
    "Charts",
    "Scheduling",
    "Tree Views",
    "Unified Field",
];

/// Legacy flat preview nav: every registration grouped by category, in
/// [`CATEGORY_ORDER`] order.
#[component]
pub fn PreviewNav() -> impl IntoView {
    let (categories, _) = signal(group_by_category(collect_preview_registrations()));

    view! {
        <For
            each=move || categories.get()
            key=|(category, _)| category.clone()
            children=|(category, items)| view! {
                <PreviewNavCategory category=category items=items />
            }
        />
    }
}

#[component]
fn PreviewNavCategory(
    /// Category to render.
    category: String,
    /// List of items to render.
    items: Vec<&'static PreviewRegistration>,
) -> impl IntoView {
    let category_icon = category_icon(&category);
    let items = StoredValue::new(items);
    let category_value = StoredValue::new(category.clone());

    view! {
        <NavigationCategory value=Signal::derive(move || category_value.get_value())>
            <NavigationCategoryHeader slot icon=category_icon>
                {category.clone()}
            </NavigationCategoryHeader>
            <NavigationSubItemGroup>
                <For
                    each=move || items.get_value()
                    key=|item| item.slug
                    children=move |item| {
                        let path = preview_page(item.slug);
                        view! {
                            <NavigationLink path=path.clone() value=path icon=item.icon exact=true>
                                {item.label}
                            </NavigationLink>
                        }
                    }
                />
            </NavigationSubItemGroup>
        </NavigationCategory>
    }
}

fn category_icon(category: &str) -> icondata_core::Icon {
    match category {
        "Layout" => icondata::AiLayoutOutlined,
        "Typography" => icondata::AiFontSizeOutlined,
        "Theme" => icondata::AiBgColorsOutlined,
        "Surfaces" => icondata::AiBorderOutlined,
        "Inputs" => icondata::AiFormOutlined,
        "Calendar & Time" => icondata::AiCalendarOutlined,
        "Data Display" => icondata::AiTableOutlined,
        "Feedback" => icondata::AiNotificationOutlined,
        "Navigation" => icondata::AiMenuOutlined,
        "Shell" => icondata::AiAppstoreOutlined,
        "Site" => icondata::AiGlobalOutlined,
        "Sections" => icondata::AiBlockOutlined,
        "Motion" => icondata::AiThunderboltOutlined,
        "Data Table" => icondata::AiDatabaseOutlined,
        "Charts" => icondata::AiBarChartOutlined,
        "Scheduling" => icondata::AiScheduleOutlined,
        "Tree Views" => icondata::AiApartmentOutlined,
        "Unified Field" => icondata::AiCloudOutlined,
        _ => icondata::AiAppstoreOutlined,
    }
}

fn group_by_category(registrations: Vec<&'static PreviewRegistration>) -> Vec<CategoryGroup> {
    let mut map: std::collections::BTreeMap<String, Vec<&PreviewRegistration>> =
        std::collections::BTreeMap::new();
    for item in registrations {
        map.entry(item.category.to_string()).or_default().push(item);
    }

    CATEGORY_ORDER
        .iter()
        .filter_map(|category| {
            map.remove(*category).map(|mut items| {
                items.sort_by(|a, b| {
                    preview_sort_key(a.category, a.slug, a.label)
                        .cmp(&preview_sort_key(b.category, b.slug, b.label))
                });
                (category.to_string(), items)
            })
        })
        .collect()
}

/// Shipped previews first, catalog stubs second; then label order.
/// Within Data Table, product entry pages sort before topic stubs.
fn preview_sort_key(category: &str, slug: &str, label: &str) -> (u8, u8, String) {
    let tier = if is_catalog_stub(category, slug) {
        1
    } else {
        0
    };
    let priority = category_entry_priority(category, slug);
    (tier, priority, label.to_ascii_lowercase())
}

fn is_catalog_stub(category: &str, slug: &str) -> bool {
    matches!(
        category,
        "Data Table" | "Charts" | "Scheduling" | "Tree Views"
    ) || (category == "Calendar & Time"
        && !matches!(slug, "date-picker" | "time-picker" | "calendar"))
        || matches!(
            slug,
            "material"
                | "backdrop"
                | "portal"
                | "to-mount-node-props"
                | "transfer-list"
                | "split-button"
                | "compound-button"
                | "floating-action-button"
                | "swatch-picker"
                | "list"
                | "avatar-group"
                | "presence-badge"
                | "counter-badge"
                | "carousel"
                | "carousel-nav"
                | "overflow"
                | "teaching-popover"
                | "bottom-navigation"
                | "toolbar"
        )
}

fn category_entry_priority(category: &str, slug: &str) -> u8 {
    match category {
        "Data Table" => match slug {
            "data-table" => 0,
            "data-table-rows" => 1,
            "data-table-columns" => 2,
            "data-table-export" => 3,
            _ => 4,
        },
        "Scheduling" => match slug {
            "scheduler-event-calendar" => 0,
            "scheduler-event-timeline" => 1,
            "scheduler-quickstart" => 2,
            _ => 4,
        },
        "Surfaces" => match slug {
            "card" => 0,
            "card-header" => 1,
            "card-preview" => 2,
            "card-footer" => 3,
            _ => 4,
        },
        "Unified Field" => match slug {
            "tag-catalog-picker" => 0,
            _ => 4,
        },
        _ => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_order_has_eighteen_entries() {
        assert_eq!(CATEGORY_ORDER.len(), 18);
    }

    #[test]
    fn thaw_calendar_is_shipped_tier() {
        assert!(!is_catalog_stub("Calendar & Time", "calendar"));
        assert!(is_catalog_stub("Calendar & Time", "date-field"));
    }

    #[test]
    fn scheduler_slugs_are_catalog_tier() {
        assert!(is_catalog_stub("Scheduling", "scheduler-event-calendar"));
    }
}
