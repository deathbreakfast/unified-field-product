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

/// Server function backing the search-source-picker preview: queries the real
/// [`uf_search_core::SearchSourceRegistry`] so the preview reflects live data.
///
/// Requires an authenticated session. Uses the request actor's Valence so
/// principal search respects viewer-scoped privacy (not System elevation).
#[server]
pub async fn preview_search_principals(
    /// Search sources to query.
    source_keys: Vec<SearchSourceKey>,
    /// Optional free-text query; empty/missing returns each source's default results.
    query: Option<String>,
    /// Maximum number of results to return per source.
    limit_per_source: u32,
) -> Result<Vec<SearchSourceItem>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        let limit_per_source = clamp_preview_search_limit(limit_per_source);

        let ctx = higgs::Higgs::from_request().await?;
        if ctx.session_user_id().is_none() {
            return Err(ServerFnError::new("You must be signed in"));
        }
        let v = ctx
            .valence()
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        let registry = uf_search_core::SearchSourceRegistry::auto_discover();
        let query_text = query.unwrap_or_default();
        return registry
            .query_many(&source_keys, &v, &query_text, limit_per_source)
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to query preview principals: {}", e)));
    }

    #[allow(unreachable_code)]
    {
        let _ = (source_keys, query, limit_per_source);
        Ok(Vec::new())
    }
}

/// Wired SearchSourcePicker preview with live query callbacks.
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
    use super::clamp_preview_search_limit;

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
}
