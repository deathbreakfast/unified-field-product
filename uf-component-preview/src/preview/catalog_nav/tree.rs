use std::collections::HashMap;

use uf_product::preview::{
    category_group_cmp, category_group_priority, category_open_key, group_open_key,
    preview_registration_cmp, section_group_cmp, section_open_key, CategoryGroup,
    PreviewRegistration, SectionGroup,
};

pub(super) const DOMAIN_SECTIONS: &[&str] = &["Charts", "Data Table", "Scheduling"];

pub(super) const SECTION_BANDS: &[(&str, u16, &str)] = &[
    ("Core Components", 10, "Essentials"),
    ("Core Components", 60, "Extended"),
];

pub(super) const PREVIEW_LINK_ICON: icondata_core::Icon = icondata::AiFileOutlined;

#[derive(Clone)]
pub(super) enum NavNode {
    Band {
        title: String,
    },
    SectionTitle {
        title: String,
        depth: u8,
        band: bool,
    },
    GroupFolder {
        open_key: String,
        title: String,
        items: Vec<&'static PreviewRegistration>,
        header_depth: u8,
        link_depth: u8,
    },
    Link {
        item: &'static PreviewRegistration,
        depth: u8,
    },
}

pub(super) fn is_domain_section(section: &str) -> bool {
    DOMAIN_SECTIONS.contains(&section)
}

pub(super) fn group_by_section(
    registrations: Vec<&'static PreviewRegistration>,
) -> Vec<SectionGroup<'static>> {
    let mut top_items: Vec<&PreviewRegistration> = registrations
        .iter()
        .copied()
        .filter(|item| item.nav_item && item.section.is_empty())
        .collect();
    top_items.sort_by(|a, b| preview_registration_cmp(a, b));

    let mut section_map: HashMap<String, (u16, Vec<&PreviewRegistration>)> = HashMap::new();
    for item in registrations {
        if item.section.is_empty() {
            continue;
        }
        section_map
            .entry(item.section.to_string())
            .or_insert((item.section_priority, Vec::new()))
            .1
            .push(item);
    }

    let mut sections: Vec<SectionGroup> = section_map
        .into_iter()
        .map(|(section, (priority, items))| {
            let mut section_nav_items: Vec<&PreviewRegistration> =
                items.iter().copied().filter(|item| item.nav_item).collect();
            section_nav_items.sort_by(|a, b| preview_registration_cmp(a, b));

            let mut category_map: HashMap<String, Vec<&PreviewRegistration>> = HashMap::new();
            for item in items.iter().copied().filter(|item| !item.nav_item) {
                let key = display_category_key(item);
                category_map.entry(key).or_default().push(item);
            }

            let mut categories: Vec<CategoryGroup> = category_map
                .into_iter()
                .map(|(category, mut items)| {
                    items.sort_by(|a, b| preview_registration_cmp(a, b));
                    (category, items)
                })
                .collect();
            categories.sort_by(|a, b| category_group_cmp(a, b));

            (section, priority, section_nav_items, categories)
        })
        .collect();

    sections.sort_by(|a, b| section_group_cmp(a, b));

    if !top_items.is_empty() {
        sections.insert(0, (String::new(), 0, top_items, Vec::new()));
    }

    sections
}

pub(super) fn is_data_table_flat_item(item: &PreviewRegistration) -> bool {
    item.section == "Data Table" && item.group.is_empty()
}

pub(super) fn display_category_key(item: &PreviewRegistration) -> String {
    if is_domain_section(item.section) && !item.group.is_empty() {
        return item.group.to_string();
    }
    if is_data_table_flat_item(item) {
        return format!("data-table::{}", item.slug);
    }
    item.category.to_string()
}

pub(super) fn should_show_category_header(
    section: &str,
    categories: &[(String, Vec<&'static PreviewRegistration>)],
) -> bool {
    if categories.is_empty() {
        return false;
    }
    if is_domain_section(section) {
        return false;
    }
    if section == "Motion" && categories.len() == 1 && categories[0].0 == "Motion" {
        return false;
    }
    if categories.len() > 1 {
        return true;
    }
    let total_links: usize = categories.iter().map(|(_, items)| items.len()).sum();
    if total_links <= 3 && section == "Tree" {
        return false;
    }
    true
}

pub(super) fn append_category_content(
    nodes: &mut Vec<NavNode>,
    section: &str,
    category: &str,
    items: &[&'static PreviewRegistration],
    base_link_depth: u8,
) {
    if is_domain_section(section) {
        if items.len() == 1 && items[0].group.is_empty() {
            nodes.push(NavNode::Link {
                item: items[0],
                depth: base_link_depth,
            });
        } else {
            nodes.push(NavNode::GroupFolder {
                open_key: category_open_key(section, category),
                title: category.to_string(),
                items: items.to_vec(),
                header_depth: 1,
                link_depth: base_link_depth + 1,
            });
        }
        return;
    }

    let mut ungrouped: Vec<&PreviewRegistration> = Vec::new();
    let mut groups: HashMap<String, Vec<&PreviewRegistration>> = HashMap::new();

    for item in items {
        if item.group.is_empty() {
            ungrouped.push(item);
        } else {
            groups.entry(item.group.to_string()).or_default().push(item);
        }
    }

    for item in ungrouped {
        nodes.push(NavNode::Link {
            item,
            depth: base_link_depth,
        });
    }

    let mut grouped: Vec<(String, Vec<&PreviewRegistration>)> = groups.into_iter().collect();
    grouped.sort_by(|a, b| {
        let a_priority = a.1.first().map(|item| item.group_priority).unwrap_or(100);
        let b_priority = b.1.first().map(|item| item.group_priority).unwrap_or(100);
        a_priority.cmp(&b_priority).then_with(|| a.0.cmp(&b.0))
    });

    for (group, group_items) in grouped {
        if group_items.len() >= 2 {
            nodes.push(NavNode::GroupFolder {
                open_key: group_open_key(section, category, &group),
                title: group,
                items: group_items,
                header_depth: base_link_depth,
                link_depth: base_link_depth + 1,
            });
        } else if let Some(item) = group_items.into_iter().next() {
            nodes.push(NavNode::Link {
                item,
                depth: base_link_depth,
            });
        }
    }
}

pub(super) fn build_section_nodes(
    section: &str,
    section_nav_items: &[&'static PreviewRegistration],
    categories: &[(String, Vec<&'static PreviewRegistration>)],
) -> Vec<NavNode> {
    let mut nodes = Vec::new();
    let mut bands: Vec<_> = SECTION_BANDS
        .iter()
        .filter(|(name, _, _)| *name == section)
        .copied()
        .collect();
    bands.sort_by_key(|(_, priority, _)| *priority);
    let mut next_band = 0;

    for item in section_nav_items {
        nodes.push(NavNode::Link { item, depth: 1 });
    }

    let show_headers = should_show_category_header(section, categories);

    for (category, items) in categories {
        let category_priority = category_group_priority(items);

        while next_band < bands.len() && category_priority >= bands[next_band].1 {
            nodes.push(NavNode::Band {
                title: bands[next_band].2.to_string(),
            });
            next_band += 1;
        }

        if show_headers {
            nodes.push(NavNode::SectionTitle {
                title: category.clone(),
                depth: 2,
                band: false,
            });
        }

        let base_link_depth = if show_headers { 2 } else { 1 };
        append_category_content(&mut nodes, section, category, items, base_link_depth);
    }

    nodes
}

pub(super) fn open_keys_for_slug(
    sections: &[SectionGroup],
    slug: &str,
) -> (Option<String>, Vec<String>) {
    for (section, _, section_nav_items, categories) in sections {
        if section_nav_items.iter().any(|item| item.slug == slug) {
            let key = section_open_key(section);
            return (Some(key.clone()), vec![key]);
        }

        for (display_category, items) in categories {
            let Some(item) = items.iter().find(|item| item.slug == slug) else {
                continue;
            };

            let section_key = section_open_key(section);
            let mut keys = vec![section_key.clone()];

            if is_domain_section(section) {
                if !item.group.is_empty() {
                    keys.push(category_open_key(section, display_category));
                }
                return (Some(section_key), keys);
            }

            if !item.group.is_empty() {
                let group_count = items
                    .iter()
                    .filter(|candidate| candidate.group == item.group)
                    .count();
                if group_count >= 2 {
                    keys.push(group_open_key(section, display_category, item.group));
                }
            }

            return (Some(section_key), keys);
        }
    }
    (None, Vec::new())
}
