use leptos::prelude::*;
use leptos_router::hooks::{use_location, use_params_map};
use uf_product::preview::{section_open_key, PreviewRegistration};
use uf_product::primitives::{
    MaterialCorners, MaterialElevation, MaterialVariant, Navigation, NavigationBody,
    NavigationCategory, NavigationCategoryHeader, NavigationConfig, NavigationDensity,
    NavigationItem, NavigationItemConfig, NavigationMaterial, NavigationSectionHeader,
    NavigationSubItem, NavigationSubItemGroup,
};

use super::super::collect_preview_registrations as collect_all_preview_registrations;
use super::super::navigation::preview_registration_href;
use super::super::slug_page::{normalize_preview_slug, preview_slug_from_route};
use super::tree::{
    build_section_nodes, group_by_section, open_keys_for_slug, NavNode, PREVIEW_LINK_ICON,
};

#[component]
fn PreviewNavItem(
    /// Item to render.
    item: &'static PreviewRegistration,
) -> impl IntoView {
    let slug = item.slug.to_string();
    let href = Signal::derive({
        let item = item;
        move || preview_registration_href(item)
    });
    let value = Signal::derive({
        let slug = slug.clone();
        move || {
            if slug.is_empty() {
                "introduction".to_string()
            } else {
                slug.clone()
            }
        }
    });

    view! {
        <NavigationItem config=NavigationItemConfig::from_signal(value).with_href(href) icon=PREVIEW_LINK_ICON>
            {item.label}
        </NavigationItem>
    }
}

#[component]
fn PreviewNavLink(
    /// Item to render.
    item: &'static PreviewRegistration,
    /// Nesting depth used for indentation.
    depth: u8,
) -> impl IntoView {
    let slug = item.slug.to_string();
    let href = Signal::derive({
        let item = item;
        move || preview_registration_href(item)
    });
    let value = Signal::derive(move || slug.clone());

    view! {
        <NavigationSubItem config=NavigationItemConfig::from_signal(value).with_href(href).with_depth(depth) icon=PREVIEW_LINK_ICON>
            {item.label}
        </NavigationSubItem>
    }
}

#[component]
fn PreviewNavSectionTitle(
    /// Title text.
    title: String,
    /// Nesting depth used for indentation.
    depth: u8,
    /// Visual band/tier to render.
    band: bool,
) -> impl IntoView {
    view! {
        <NavigationSectionHeader depth=depth band=band>
            {title}
        </NavigationSectionHeader>
    }
}

#[component]
fn PreviewNavGroupFolder(
    /// Open key.
    open_key: String,
    /// Title text.
    title: String,
    /// List of items to render.
    items: Vec<&'static PreviewRegistration>,
    /// Header depth.
    header_depth: u8,
    /// Link depth.
    link_depth: u8,
) -> impl IntoView {
    let category_value = StoredValue::new(open_key);
    let folder_items = StoredValue::new(items);

    view! {
        <NavigationCategory value=Signal::derive(move || category_value.get_value())>
            <NavigationCategoryHeader slot icon=icondata::AiFolderOutlined depth=header_depth>
                {title}
            </NavigationCategoryHeader>
            <NavigationSubItemGroup>
                <For
                    each=move || folder_items.get_value()
                    key=|item| item.slug
                    children=move |item| view! { <PreviewNavLink item depth=link_depth /> }
                />
            </NavigationSubItemGroup>
        </NavigationCategory>
    }
}

#[component]
fn PreviewNavSection(
    /// Section to render.
    section: String,
    /// List of section nav items.
    section_nav_items: Vec<&'static PreviewRegistration>,
    /// List of categories to render.
    categories: Vec<(String, Vec<&'static PreviewRegistration>)>,
) -> impl IntoView {
    let section_value = StoredValue::new(section_open_key(&section));
    let nodes = StoredValue::new(build_section_nodes(
        &section,
        &section_nav_items,
        &categories,
    ));

    view! {
        <NavigationCategory value=Signal::derive(move || section_value.get_value())>
            <NavigationCategoryHeader slot section_folder=true icon=icondata::AiFolderOutlined>
                {section.clone()}
            </NavigationCategoryHeader>
            <NavigationSubItemGroup>
                <For
                    each=move || nodes.get_value()
                    key=|node| match node {
                        NavNode::Band { title } => format!("band:{title}"),
                        NavNode::SectionTitle { title, depth, .. } => {
                            format!("title:{depth}:{title}")
                        }
                        NavNode::GroupFolder { open_key, title, .. } => {
                            format!("folder:{open_key}:{title}")
                        }
                        NavNode::Link { item, depth } => format!("link:{depth}:{}", item.slug),
                    }
                    children=|node| match node {
                        NavNode::Band { title } => {
                            view! { <PreviewNavSectionTitle title=title depth=2 band=true /> }.into_any()
                        }
                        NavNode::SectionTitle { title, depth, band } => {
                            view! { <PreviewNavSectionTitle title=title depth=depth band=band /> }.into_any()
                        }
                        NavNode::GroupFolder {
                            open_key,
                            title,
                            items,
                            header_depth,
                            link_depth,
                        } => {
                            view! {
                                <PreviewNavGroupFolder
                                    open_key=open_key
                                    title=title
                                    items=items
                                    header_depth=header_depth
                                    link_depth=link_depth
                                />
                            }
                                .into_any()
                        }
                        NavNode::Link { item, depth } => {
                            view! { <PreviewNavLink item depth /> }.into_any()
                        }
                    }
                />
            </NavigationSubItemGroup>
        </NavigationCategory>
    }
}

/// Sidebar navigation for the preview catalog shell: registrations grouped by
/// section/category/group, matching the current route.
#[component]
pub fn PreviewCatalogNav() -> impl IntoView {
    let sections = StoredValue::new(group_by_section(collect_all_preview_registrations()));
    let params = use_params_map();
    let location = use_location();

    let initial_slug = normalize_preview_slug(&preview_slug_from_route(
        &params.get_untracked(),
        &location.pathname.get_untracked(),
    ));

    let selected_value = RwSignal::new(if initial_slug.is_empty() {
        None
    } else {
        Some(initial_slug.clone())
    });

    let (initial_section, initial_open) = open_keys_for_slug(&sections.get_value(), &initial_slug);
    let selected_category_value = RwSignal::new(initial_section);

    // Only ancestors of the current slug start open — no section/group default-open.
    let open_categories = RwSignal::new(initial_open);

    #[cfg(not(feature = "ssr"))]
    Effect::new(move |_| {
        let slug = normalize_preview_slug(&preview_slug_from_route(
            &params.get(),
            &location.pathname.get(),
        ));
        if slug.is_empty() {
            selected_value.set(None);
            selected_category_value.set(None);
            open_categories.set(Vec::new());
            return;
        }

        selected_value.set(Some(slug.clone()));
        let (section, keys) = open_keys_for_slug(&sections.get_value(), &slug);
        selected_category_value.set(section);
        open_categories.set(keys);
    });

    view! {
        <Navigation
            data_testid="preview-catalog-nav"
            config=NavigationConfig::new()
                .with_selected_value(selected_value)
                .with_selected_category_value(selected_category_value)
                .with_open_categories(open_categories)
                .with_density(NavigationDensity::Compact)
        >
            <NavigationMaterial
                variant=MaterialVariant::Solid
                elevation=MaterialElevation::Flat
                corners=MaterialCorners::Square
                slot
            />
            <NavigationBody slot>
                <For
                    each=move || sections.get_value()
                    key=|(section, _, _, _)| section.clone()
                    children=|(section, _, section_nav_items, categories)| {
                        if section.is_empty() {
                            view! {
                                <For
                                    each=move || section_nav_items.clone()
                                    key=|item| item.slug
                                    children=|item| view! { <PreviewNavItem item /> }
                                />
                            }.into_any()
                        } else {
                            view! {
                                <PreviewNavSection
                                    section=section
                                    section_nav_items=section_nav_items
                                    categories=categories
                                />
                            }.into_any()
                        }
                    }
                />
            </NavigationBody>
        </Navigation>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icondata::AiAppstoreOutlined;

    static CARD_HEADER: PreviewRegistration = PreviewRegistration {
        slug: "card-header",
        label: "card-header",
        section: "Core Components",
        section_priority: 2,
        category: "Surfaces",
        category_priority: 20,
        category_default_collapsed: true,
        group: "Card",
        group_priority: 10,
        nav_item: false,
        icon: AiAppstoreOutlined,
        render: || view! { <span></span> }.into_any(),
    };

    static CARD_FOOTER: PreviewRegistration = PreviewRegistration {
        slug: "card-footer",
        label: "card-footer",
        section: "Core Components",
        section_priority: 2,
        category: "Surfaces",
        category_priority: 20,
        category_default_collapsed: true,
        group: "Card",
        group_priority: 10,
        nav_item: false,
        icon: AiAppstoreOutlined,
        render: || view! { <span></span> }.into_any(),
    };

    static MATERIAL: PreviewRegistration = PreviewRegistration {
        slug: "material",
        label: "material",
        section: "Core Components",
        section_priority: 2,
        category: "Surfaces",
        category_priority: 20,
        category_default_collapsed: true,
        group: "",
        group_priority: 0,
        nav_item: false,
        icon: AiAppstoreOutlined,
        render: || view! { <span></span> }.into_any(),
    };

    static BAR_CHART: PreviewRegistration = PreviewRegistration {
        slug: "bar-chart",
        label: "bar-chart",
        section: "Charts",
        section_priority: 5,
        category: "Charts",
        category_priority: 100,
        category_default_collapsed: true,
        group: "Chart Types",
        group_priority: 10,
        nav_item: false,
        icon: AiAppstoreOutlined,
        render: || view! { <span></span> }.into_any(),
    };

    static DATA_TABLE_COLUMN: PreviewRegistration = PreviewRegistration {
        slug: "data-table-columns",
        label: "Column Features",
        section: "Data Table",
        section_priority: 6,
        category: "Data Table",
        category_priority: 100,
        category_default_collapsed: true,
        group: "Columns",
        group_priority: 20,
        nav_item: false,
        icon: AiAppstoreOutlined,
        render: || view! { <span></span> }.into_any(),
    };

    static DATA_TABLE_OVERVIEW: PreviewRegistration = PreviewRegistration {
        slug: "data-table",
        label: "Data Table",
        section: "Data Table",
        section_priority: 6,
        category: "Data Table",
        category_priority: 100,
        category_default_collapsed: true,
        group: "",
        group_priority: 10,
        nav_item: false,
        icon: AiAppstoreOutlined,
        render: || view! { <span></span> }.into_any(),
    };

    static DATA_TABLE_ROWS: PreviewRegistration = PreviewRegistration {
        slug: "data-table-rows",
        label: "Rows",
        section: "Data Table",
        section_priority: 6,
        category: "Data Table",
        category_priority: 100,
        category_default_collapsed: true,
        group: "",
        group_priority: 30,
        nav_item: false,
        icon: AiAppstoreOutlined,
        render: || view! { <span></span> }.into_any(),
    };

    static DATA_TABLE_EXPORT: PreviewRegistration = PreviewRegistration {
        slug: "data-table-export",
        label: "Export & Clipboard",
        section: "Data Table",
        section_priority: 6,
        category: "Data Table",
        category_priority: 100,
        category_default_collapsed: true,
        group: "Selection & IO",
        group_priority: 70,
        nav_item: false,
        icon: AiAppstoreOutlined,
        render: || view! { <span></span> }.into_any(),
    };

    static DATE_PICKER: PreviewRegistration = PreviewRegistration {
        slug: "date-picker",
        label: "date-picker",
        section: "Core Components",
        section_priority: 2,
        category: "Calendar & Time",
        category_priority: 70,
        category_default_collapsed: true,
        group: "Pickers",
        group_priority: 10,
        nav_item: false,
        icon: AiAppstoreOutlined,
        render: || view! { <span></span> }.into_any(),
    };

    static CALENDAR: PreviewRegistration = PreviewRegistration {
        slug: "calendar",
        label: "calendar",
        section: "Core Components",
        section_priority: 2,
        category: "Calendar & Time",
        category_priority: 70,
        category_default_collapsed: true,
        group: "Pickers",
        group_priority: 10,
        nav_item: false,
        icon: AiAppstoreOutlined,
        render: || view! { <span></span> }.into_any(),
    };

    fn node_slugs(nodes: &[NavNode]) -> Vec<&str> {
        nodes
            .iter()
            .filter_map(|node| match node {
                NavNode::Link { item, .. } => Some(item.slug),
                _ => None,
            })
            .collect()
    }

    fn has_group_folder(nodes: &[NavNode], title: &str) -> bool {
        nodes.iter().any(|node| match node {
            NavNode::GroupFolder {
                title: group_title, ..
            } => group_title == title,
            _ => false,
        })
    }

    #[test]
    fn motion_section_hides_redundant_category_header() {
        static MOTION_OVERVIEW: PreviewRegistration = PreviewRegistration {
            slug: "motion",
            label: "Overview",
            section: "Motion",
            section_priority: 3,
            category: "Motion",
            category_priority: 100,
            category_default_collapsed: true,
            group: "Overview",
            group_priority: 10,
            nav_item: false,
            icon: AiAppstoreOutlined,
            render: || view! { <span></span> }.into_any(),
        };

        static MOTION_ATOMS: PreviewRegistration = PreviewRegistration {
            slug: "motion-atoms",
            label: "Motion Atoms",
            section: "Motion",
            section_priority: 3,
            category: "Motion",
            category_priority: 100,
            category_default_collapsed: true,
            group: "Atoms",
            group_priority: 20,
            nav_item: false,
            icon: AiAppstoreOutlined,
            render: || view! { <span></span> }.into_any(),
        };

        let sections = group_by_section(vec![&MOTION_OVERVIEW, &MOTION_ATOMS]);
        let (_, _, _, categories) = sections
            .iter()
            .find(|(section, _, _, _)| section == "Motion")
            .expect("motion section");
        let nodes = build_section_nodes("Motion", &[], categories);
        assert!(!nodes.iter().any(|node| matches!(
            node,
            NavNode::SectionTitle { title, .. } if title == "Motion"
        )));
    }

    #[test]
    fn surfaces_card_group_renders_as_folder() {
        let categories = vec![(
            "Surfaces".to_string(),
            vec![&CARD_HEADER, &CARD_FOOTER, &MATERIAL],
        )];
        let nodes = build_section_nodes("Core Components", &[], &categories);
        assert!(has_group_folder(&nodes, "Card"));
        assert!(node_slugs(&nodes).contains(&"material"));
    }

    #[test]
    fn open_keys_for_unselected_slug_opens_nothing() {
        let sections = group_by_section(vec![&CARD_HEADER, &CARD_FOOTER, &DATE_PICKER, &BAR_CHART]);
        let (section, keys) = open_keys_for_slug(&sections, "");
        assert!(section.is_none());
        assert!(keys.is_empty());
    }

    #[test]
    fn open_keys_for_card_header_opens_section_and_group() {
        let sections = group_by_section(vec![&CARD_HEADER, &CARD_FOOTER]);
        let (section, keys) = open_keys_for_slug(&sections, "card-header");
        assert_eq!(section.as_deref(), Some("core-components"));
        assert!(keys.contains(&"core-components".to_string()));
        assert!(keys.contains(&"core-components/surfaces/card".to_string()));
    }

    #[test]
    fn domain_section_renders_group_folders_without_category_header() {
        let categories = vec![("Chart Types".to_string(), vec![&BAR_CHART])];
        let nodes = build_section_nodes("Charts", &[], &categories);
        assert!(has_group_folder(&nodes, "Chart Types"));
        assert!(!nodes.iter().any(|node| matches!(
            node,
            NavNode::SectionTitle { title, .. } if title == "Chart Types"
        )));
    }

    #[test]
    fn data_table_section_renders_flat_links_and_multi_item_folders() {
        let sections = group_by_section(vec![
            &DATA_TABLE_OVERVIEW,
            &DATA_TABLE_ROWS,
            &DATA_TABLE_COLUMN,
            &DATA_TABLE_EXPORT,
        ]);
        let (_, _, _, categories) = sections
            .iter()
            .find(|(section, _, _, _)| section == "Data Table")
            .expect("data table section");
        let nodes = build_section_nodes("Data Table", &[], categories);

        assert_eq!(node_slugs(&nodes), vec!["data-table", "data-table-rows"]);
        assert!(has_group_folder(&nodes, "Columns"));
        assert!(has_group_folder(&nodes, "Selection & IO"));
    }

    #[test]
    fn open_keys_for_data_table_column_opens_section_and_group_folder() {
        let sections = group_by_section(vec![&DATA_TABLE_COLUMN]);
        let (section, keys) = open_keys_for_slug(&sections, "data-table-columns");
        assert_eq!(section.as_deref(), Some("data-table"));
        assert!(keys.contains(&"data-table".to_string()));
        assert!(keys.contains(&"data-table/columns".to_string()));
    }

    #[test]
    fn open_keys_for_data_table_overview_opens_only_section() {
        let sections = group_by_section(vec![&DATA_TABLE_OVERVIEW]);
        let (section, keys) = open_keys_for_slug(&sections, "data-table");
        assert_eq!(section.as_deref(), Some("data-table"));
        assert_eq!(keys, vec!["data-table".to_string()]);
    }

    #[test]
    fn open_keys_for_calendar_picker_opens_pickers_folder() {
        let sections = group_by_section(vec![&DATE_PICKER, &CALENDAR]);
        let (_, keys) = open_keys_for_slug(&sections, "date-picker");
        assert!(keys.contains(&"core-components/calendar-and-time/pickers".to_string()));
    }
}
