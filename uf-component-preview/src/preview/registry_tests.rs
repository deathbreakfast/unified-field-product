#[cfg(all(test, feature = "ssr"))]
mod tests {
    use crate::preview::collect_preview_registrations;

    const PLACEHOLDER_SLUGS: &[&str] = &[
        "avatar-group",
        "carousel",
        "compound-button",
        "counter-badge",
        "data-table",
        "demo-status-pill",
        "list",
        "overflow",
        "presence-badge",
        "swatch-picker",
        "toolbar",
    ];

    #[test]
    fn placeholder_preview_slugs_are_registered() {
        let slugs: Vec<_> = collect_preview_registrations()
            .iter()
            .map(|item| item.slug)
            .collect();

        for slug in PLACEHOLDER_SLUGS {
            assert!(
                slugs.contains(&slug),
                "missing preview registration for `{slug}` (have {} slugs)",
                slugs.len()
            );
        }
    }
}
