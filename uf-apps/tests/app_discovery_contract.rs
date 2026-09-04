//! Integration contracts for `uf_app!` registry discovery + `UfAppsRoutes` server fns.
#![allow(missing_docs)]

#[cfg(feature = "ssr")]
mod tests {
    use std::collections::HashSet;
    use uf_apps::server::{
        collect_registered_apps, filter_apps_by_query, find_app_overview, get_app_overview,
        get_apps, get_apps_page, page_apps, sort_apps_by_name, APPS_PAGE_SIZE,
    };

    #[test]
    fn collect_registered_apps_non_empty_happy_path() {
        let apps = collect_registered_apps();
        assert!(
            !apps.is_empty(),
            "uf_app! inventory should expose at least one app, got {}",
            apps.len()
        );
        assert!(
            apps.iter().all(|a| a.route_path.starts_with('/')),
            "all discovered apps should expose an absolute route path"
        );
    }

    #[test]
    fn collect_registered_apps_includes_uf_apps_routes_happy_path() {
        let apps = collect_registered_apps();
        let apps_entry = apps
            .iter()
            .find(|a| a.slug == "apps")
            .expect("uf-apps uf_app! registration (UfAppsRoutes) should be discoverable");
        assert_eq!(apps_entry.route_path, "/apps");
        assert_ne!(apps_entry.name, "");
        assert_ne!(apps_entry.description, "");
    }

    #[test]
    fn collect_registered_apps_unique_slugs_happy_path() {
        let apps = collect_registered_apps();
        let mut seen = HashSet::new();
        for app in &apps {
            assert!(
                seen.insert(app.slug.as_str()),
                "duplicate app slug: {}",
                app.slug
            );
            assert!(
                app.slug
                    .chars()
                    .all(|c: char| c.is_alphanumeric() || c == '-'),
                "slug should be URL-safe: {}",
                app.slug
            );
        }
    }

    #[tokio::test]
    async fn get_apps_mirrors_registry_happy_path() {
        let discovered = collect_registered_apps();
        let apps = get_apps().await.expect("get_apps should succeed");
        assert_eq!(apps.len(), discovered.len());
        assert_ne!(apps.len(), 0);
    }

    #[tokio::test]
    async fn get_app_overview_matches_directory_item_happy_path() {
        let apps = collect_registered_apps();
        let target = apps
            .first()
            .expect("collect_registered_apps should return at least one app");
        let overview = get_app_overview(target.slug.clone())
            .await
            .expect("get_app_overview should succeed")
            .expect("Should find discovered app");
        assert_eq!(overview.name, target.name);
        assert_eq!(overview.description, target.description);
        assert_eq!(overview.route_path, target.route_path);
    }

    #[tokio::test]
    async fn get_app_overview_unknown_slug_none_sad() {
        let overview = get_app_overview("non-existent-app".to_string())
            .await
            .expect("get_app_overview should succeed");
        assert!(
            overview.is_none(),
            "unknown slug must return None, got {overview:?}"
        );
    }

    #[tokio::test]
    async fn get_apps_page_first_page_reports_total_happy_path() {
        let discovered = collect_registered_apps();
        let page = get_apps_page(0, APPS_PAGE_SIZE, None)
            .await
            .expect("get_apps_page should succeed");
        assert_eq!(page.total_count, Some(discovered.len() as u64));
        assert!(page.items.len() <= APPS_PAGE_SIZE as usize);
        assert_eq!(
            page.items.len(),
            discovered.len().min(APPS_PAGE_SIZE as usize)
        );
    }

    #[tokio::test]
    async fn get_apps_page_query_filters_by_name_happy_path() {
        let apps = collect_registered_apps();
        let target = apps
            .first()
            .expect("collect_registered_apps should return at least one app");
        let needle = target
            .name
            .chars()
            .take(3)
            .collect::<String>()
            .to_lowercase();
        assert!(!needle.is_empty(), "target app name should be non-empty");

        let page = get_apps_page(0, APPS_PAGE_SIZE, Some(needle.clone()))
            .await
            .expect("get_apps_page should succeed");
        assert!(
            !page.items.is_empty(),
            "query `{needle}` should match at least one app"
        );
        assert!(page.items.iter().all(|a| {
            a.name.to_lowercase().contains(&needle)
                || a.description.to_lowercase().contains(&needle)
        }));
    }

    #[tokio::test]
    async fn get_apps_page_unknown_query_empty_sad() {
        let page = get_apps_page(0, APPS_PAGE_SIZE, Some("zz-no-such-product-app".into()))
            .await
            .expect("get_apps_page should succeed");
        assert_eq!(page.total_count, Some(0));
        assert_eq!(page.items.len(), 0);
    }

    #[tokio::test]
    async fn apps_directory_workflow_list_filter_overview_happy_path() {
        let mut apps = collect_registered_apps();
        assert_ne!(apps.len(), 0, "registry must be non-empty");
        sort_apps_by_name(&mut apps);

        let target = apps
            .iter()
            .find(|a| a.slug == "apps")
            .cloned()
            .or_else(|| apps.first().cloned())
            .expect("at least one app");

        let mut filtered = apps.clone();
        filter_apps_by_query(&mut filtered, Some(&target.name));
        assert!(
            filtered.iter().any(|a| a.slug == target.slug),
            "filter by name should keep the target app"
        );

        let page = page_apps(filtered, 0, APPS_PAGE_SIZE);
        assert!(page.total_count.unwrap_or(0) >= 1);

        let overview = find_app_overview(&apps, &target.slug).expect("overview for target");
        assert_eq!(overview.route_path, target.route_path);

        let via_server = get_app_overview(target.slug.clone())
            .await
            .expect("server overview")
            .expect("registered slug");
        assert_eq!(via_server.name, overview.name);
        assert_eq!(via_server.route_path, overview.route_path);
    }
}
