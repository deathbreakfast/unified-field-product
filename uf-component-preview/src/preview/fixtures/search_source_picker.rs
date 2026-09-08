use leptos::prelude::*;
use leptos::task::spawn_local;
use orbital_core_components::Body1;
use orbital_primitives::{Flex, FlexGap};
use uf_integrations::SearchSourcePicker;
use uf_search_core::{SearchSourceItem, SearchSourceKey};

/// Cap client-requested per-source fan-out for preview principal search.
pub(crate) fn clamp_preview_search_limit(limit_per_source: u32) -> u32 {
    const MAX_LIMIT_PER_SOURCE: u32 = 50;
    limit_per_source.min(MAX_LIMIT_PER_SOURCE)
}

/// Fixed mock principals for the catalog preview (no live Valence / email reads).
fn mock_preview_principals(query: Option<&str>, limit_per_source: u32) -> Vec<SearchSourceItem> {
    let limit = clamp_preview_search_limit(limit_per_source) as usize;
    let q = query.unwrap_or("").trim().to_lowercase();
    let all = [
        SearchSourceItem {
            source_id: "user_search_source".to_string(),
            id: "preview-user-ada".to_string(),
            title: "Ada Lovelace (ada001)".to_string(),
            description: Some("User account".to_string()),
            kind: "user".to_string(),
        },
        SearchSourceItem {
            source_id: "user_search_source".to_string(),
            id: "preview-user-grace".to_string(),
            title: "Grace Hopper (grace0)".to_string(),
            description: Some("User account".to_string()),
            kind: "user".to_string(),
        },
        SearchSourceItem {
            source_id: "permission_group_search_source".to_string(),
            id: "preview-group-ops".to_string(),
            title: "Ops Reviewers".to_string(),
            description: Some("Mock permission group".to_string()),
            kind: "permission_group".to_string(),
        },
        SearchSourceItem {
            source_id: "permission_group_search_source".to_string(),
            id: "preview-group-readers".to_string(),
            title: "Docs Readers".to_string(),
            description: Some("Mock permission group".to_string()),
            kind: "permission_group".to_string(),
        },
    ];
    all.into_iter()
        .filter(|item| {
            if q.is_empty() {
                return true;
            }
            item.title.to_lowercase().contains(&q)
                || item.id.to_lowercase().contains(&q)
                || item
                    .description
                    .as_ref()
                    .is_some_and(|d| d.to_lowercase().contains(&q))
        })
        .take(limit.max(1).saturating_mul(2))
        .collect()
}

/// Server function backing the search-source-picker preview with **mock** data.
///
/// Does not query live Valence or the real [`uf_search_core::SearchSourceRegistry`].
/// The `/orbital` catalog stays mounted; this fixture only demonstrates the picker UI.
#[server]
pub async fn preview_search_principals(
    /// Search sources to query (ignored for mock data; kept for API compatibility).
    source_keys: Vec<SearchSourceKey>,
    /// Optional free-text query; empty/missing returns each source's default results.
    query: Option<String>,
    /// Maximum number of results to return per source.
    limit_per_source: u32,
) -> Result<Vec<SearchSourceItem>, ServerFnError> {
    let _ = source_keys;
    Ok(mock_preview_principals(query.as_deref(), limit_per_source))
}

/// Wired SearchSourcePicker preview with mock query callbacks.
#[component]
pub fn SearchSourcePickerPreviewFixture() -> impl IntoView {
    let options = RwSignal::new(Vec::<SearchSourceItem>::new());
    let selected = RwSignal::new(Vec::<SearchSourceItem>::new());
    let error = RwSignal::new(None::<String>);

    let request_initial = Callback::new(move |sources: Vec<SearchSourceKey>| {
        spawn_local(async move {
            match preview_search_principals(sources, None, 20).await {
                Ok(rows) => options.set(rows),
                Err(err) => error.set(Some(err.to_string())),
            }
        });
    });

    let request_search = Callback::new(move |(sources, query): (Vec<SearchSourceKey>, String)| {
        spawn_local(async move {
            match preview_search_principals(sources, Some(query), 20).await {
                Ok(rows) => options.set(rows),
                Err(err) => error.set(Some(err.to_string())),
            }
        });
    });

    let on_select = Callback::new(move |item: SearchSourceItem| {
        selected.update(|items| {
            if !items
                .iter()
                .any(|existing| existing.id == item.id && existing.source_id == item.source_id)
            {
                items.push(item);
            }
        });
    });

    view! {
        <SearchSourcePicker
            search_sources=Signal::derive(|| vec![
                SearchSourceKey::new("user_search_source", "Users"),
                SearchSourceKey::new("permission_group_search_source", "Permission Groups"),
            ])
            options=options
            multiselect=true
            on_request_initial=request_initial
            on_search=request_search
            on_select=on_select
        />
    }
}

/// Selected principals list for the secondary preview card.
#[component]
pub fn SearchSourcePickerSelectedPreview() -> impl IntoView {
    let options = RwSignal::new(Vec::<SearchSourceItem>::new());
    let selected = RwSignal::new(Vec::<SearchSourceItem>::new());
    let error = RwSignal::new(None::<String>);

    let request_initial = Callback::new(move |sources: Vec<SearchSourceKey>| {
        spawn_local(async move {
            match preview_search_principals(sources, None, 20).await {
                Ok(rows) => options.set(rows),
                Err(err) => error.set(Some(err.to_string())),
            }
        });
    });

    let request_search = Callback::new(move |(sources, query): (Vec<SearchSourceKey>, String)| {
        spawn_local(async move {
            match preview_search_principals(sources, Some(query), 20).await {
                Ok(rows) => options.set(rows),
                Err(err) => error.set(Some(err.to_string())),
            }
        });
    });

    let on_select = Callback::new(move |item: SearchSourceItem| {
        selected.update(|items| {
            if !items
                .iter()
                .any(|existing| existing.id == item.id && existing.source_id == item.source_id)
            {
                items.push(item);
            }
        });
    });

    view! {
        <Flex vertical=true gap=FlexGap::Small>
            <SearchSourcePicker
                search_sources=Signal::derive(|| vec![
                    SearchSourceKey::new("user_search_source", "Users"),
                    SearchSourceKey::new("permission_group_search_source", "Permission Groups"),
                ])
                options=options
                multiselect=true
                on_request_initial=request_initial
                on_search=request_search
                on_select=on_select
            />
            <Show when=move || error.get().is_some()>
                <Body1>{move || format!("Query error: {}", error.get().unwrap_or_default())}</Body1>
            </Show>
            <For
                each=move || selected.get()
                key=|item| format!("{}:{}", item.source_id, item.id)
                let:item
            >
                <Body1>{format!("{} ({})", item.title, item.kind)}</Body1>
            </For>
        </Flex>
    }
}

/// Default preview page wrapper for the search source picker catalog entry.
#[component]
pub fn SearchSourcePickerPreview() -> impl IntoView {
    view! {
        <SearchSourcePickerPreviewFixture />
    }
}

#[cfg(test)]
mod clamp_tests {
    use super::{clamp_preview_search_limit, mock_preview_principals};

    #[test]
    fn clamp_preview_search_limit_caps_high_values_sad() {
        assert_eq!(clamp_preview_search_limit(u32::MAX), 50);
        assert_eq!(clamp_preview_search_limit(51), 50);
    }

    #[test]
    fn clamp_preview_search_limit_keeps_reasonable_happy_path() {
        assert_eq!(clamp_preview_search_limit(20), 20);
        assert_eq!(clamp_preview_search_limit(0), 0);
    }

    #[test]
    fn mock_preview_principals_filters_by_query() {
        let rows = mock_preview_principals(Some("ada"), 20);
        assert!(rows.iter().any(|r| r.id.contains("ada")));
        assert!(!rows.iter().any(|r| r.id.contains("grace")));
    }
}
