//! Multi-source search combobox over `uf-search-core` keys and items.
//!
//! Parents supply fetching: register providers once (macros / Quark on SSR), then
//! fill [`SearchSourcePicker`] options from a server fn that calls
//! `SearchSourceRegistry::query_many`. Provider registry and Valence queries
//! live in `uf-search-core`; `define_search_sources!` registration in
//! `uf-product-macros`.
//!
//! # Concern → API
//!
//! | Concern | API |
//! |---------|-----|
//! | Multi-source principal / resource picker | [`SearchSourcePicker`] |
//!
//! Detailed call shape: the component `# Examples` below. Registry contracts:
//! `uf-search-core`. Runnable shell chrome: `examples/shell-chrome-host`.

use leptos::prelude::*;

use orbital_primitives::{
    Combobox, ComboboxAppearance, ComboboxBind, ComboboxOption, ComboboxOptionGroup, Flex, FlexGap,
};
use uf_product::components::{Body1, Caption1};
use uf_search_core::{SearchSourceItem, SearchSourceKey};

type SearchSourceGroup = (String, String, Vec<SearchSourceItem>);

fn group_signature(group: &SearchSourceGroup) -> String {
    let (source_id, _, items) = group;
    let item_signature = items
        .iter()
        .map(|item| format!("{}:{}", item.source_id, item.id))
        .collect::<Vec<_>>()
        .join("|");
    format!("{source_id}|{item_signature}")
}

fn option_key(item: &SearchSourceItem) -> String {
    format!("{}:{}", item.source_id, item.id)
}

fn group_options(
    sources: Vec<SearchSourceKey>,
    rows: Vec<SearchSourceItem>,
) -> Vec<SearchSourceGroup> {
    let mut groups = Vec::<SearchSourceGroup>::new();

    for source in sources {
        let source_id = source.id;
        let source_rows = rows
            .iter()
            .filter(|item| item.source_id == source_id)
            .cloned()
            .collect::<Vec<_>>();
        if !source_rows.is_empty() {
            groups.push((source_id, source.label, source_rows));
        }
    }

    let mut other_rows = rows
        .into_iter()
        .filter(|item| !groups.iter().any(|(id, _, _)| *id == item.source_id))
        .collect::<Vec<_>>();
    if !other_rows.is_empty() {
        other_rows.sort_by(|a, b| a.source_id.cmp(&b.source_id).then(a.title.cmp(&b.title)));
        groups.push(("other".to_string(), "Other".to_string(), other_rows));
    }

    groups
}

#[component]
fn SearchSourceOptionRow(item: SearchSourceItem) -> impl IntoView {
    let key = option_key(&item);
    let title = item.title.clone();
    let kind = item.kind.clone();
    let description = item
        .description
        .clone()
        .unwrap_or_else(|| item.source_id.clone());
    view! {
        <ComboboxOption text=title.clone() value=key>
            <Flex vertical=true gap=FlexGap::Small>
                <Body1>{title}</Body1>
                <Caption1>{format!("{kind} • {description}")}</Caption1>
            </Flex>
        </ComboboxOption>
    }
}

/// Search Source Picker
///
/// Reusable principal search picker with support for one or more backend search
/// sources. This component renders an Orbital [`Combobox`] and delegates fetching
/// to parent callbacks so product apps can register backend sources once (via
/// Quark on SSR) and wire the UI with minimal frontend code.
///
/// The picker requests:
/// - initial results when mounted/opened,
/// - filtered results as the user types.
///
/// # Examples
///
/// ```rust,ignore
/// use leptos::prelude::*;
/// use uf_integrations::SearchSourcePicker;
/// use uf_search_core::{SearchSourceItem, SearchSourceKey};
///
/// #[component]
/// fn UserPicker() -> impl IntoView {
///     let sources = Signal::derive(|| vec![SearchSourceKey::new("users", "Users")]);
///     let options = RwSignal::new(Vec::<SearchSourceItem>::new());
///     view! {
///         <SearchSourcePicker
///             search_sources=sources
///             options=options.into()
///             placeholder="Search users…"
///             on_search=Callback::new(move |(keys, q)| {
///                 // Call your server fn → SearchSourceRegistry::query_many
///                 let _ = (keys, q);
///             })
///             on_select=Callback::new(move |item| { let _ = item; })
///         />
///     }
/// }
/// ```
#[component]
pub fn SearchSourcePicker(
    /// Backend source keys to query.
    #[prop(into)]
    search_sources: Signal<Vec<SearchSourceKey>>,
    /// Current option rows to render.
    #[prop(into)]
    options: Signal<Vec<SearchSourceItem>>,
    /// Placeholder text for the search input.
    #[prop(optional, into)]
    placeholder: MaybeProp<String>,
    /// Enable selecting multiple options. Defaults to `false` (single-select).
    #[prop(default = false)]
    multiselect: bool,
    /// Callback fired on mount to request initial results.
    #[prop(optional)]
    on_request_initial: Option<Callback<Vec<SearchSourceKey>>>,
    /// Callback fired when search text changes.
    #[prop(optional)]
    on_search: Option<Callback<(Vec<SearchSourceKey>, String)>>,
    /// Callback fired when an option is selected.
    #[prop(optional)]
    on_select: Option<Callback<SearchSourceItem>>,
    /// Externally-controlled current selection. When `Some`, the picker syncs
    /// its displayed query and internal selected key to this item; useful for
    /// restoring a parent-owned selection across mount/unmount cycles.
    #[prop(optional, into)]
    value: MaybeProp<SearchSourceItem>,
    /// When true, the control is non-interactive (matches disabled app-bar utilities).
    #[prop(default = false)]
    disabled: bool,
) -> impl IntoView {
    let query = RwSignal::new(String::new());
    let selected_options = RwSignal::new(Vec::<String>::new());
    let last_emitted_selection = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        if let Some(cb) = &on_request_initial {
            cb.run(search_sources.get());
        }
    });

    // Sync external `value` into internal display state.
    Effect::new(move |_| {
        let next = value.get();
        let next_key = next.as_ref().map(option_key);
        let current = selected_options.get_untracked();
        let current_key = current.first().cloned();
        if current_key == next_key {
            return;
        }
        if let Some(item) = next {
            selected_options.set(vec![option_key(&item)]);
            query.set(item.title);
            last_emitted_selection.set(next_key);
        } else {
            selected_options.set(Vec::new());
            query.set(String::new());
            last_emitted_selection.set(None);
        }
    });

    // Emit search callbacks when typed text changes (skip the initial empty mount).
    let search_ready = StoredValue::new(false);
    Effect::new(move |_| {
        let next_query = query.get();
        if !search_ready.get_value() {
            search_ready.set_value(true);
            return;
        }
        if let Some(cb) = &on_search {
            cb.run((search_sources.get_untracked(), next_query));
        }
    });

    // Emit selection when Combobox selected_options change.
    Effect::new(move |_| {
        let keys = selected_options.get();
        let Some(selected_key) = keys.last().cloned() else {
            return;
        };
        if last_emitted_selection.get_untracked().as_ref() == Some(&selected_key) {
            return;
        }
        let rows = options.get_untracked();
        if let Some(item) = rows
            .into_iter()
            .find(|item| option_key(item) == selected_key)
        {
            last_emitted_selection.set(Some(selected_key));
            if let Some(cb) = &on_select {
                cb.run(item);
            }
        }
    });

    let placeholder_text = move || {
        placeholder
            .get()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "Search users or groups".to_string())
    };

    let grouped_options = Memo::new(move |_| group_options(search_sources.get(), options.get()));

    let appearance = ComboboxAppearance {
        placeholder: MaybeProp::from(Signal::derive(placeholder_text)),
        disabled: Signal::from(disabled),
        clearable: true,
        multiselect: Signal::from(multiselect),
        ..Default::default()
    };

    // Orbital escape: Combobox needs a min-width shell; no Flex/width token covers
    // this command-palette sizing yet.
    view! {
        <div data-testid="search-source-picker" style="min-width: 250px; position: relative;">
            <Combobox
                bind=ComboboxBind::new(query, selected_options)
                appearance=appearance
            >
                <For
                    each=move || grouped_options.get()
                    key=group_signature
                    let:group
                >
                    {
                        let (_, label, items) = group;
                        view! {
                            <ComboboxOptionGroup label=label>
                                <For
                                    each=move || items.clone()
                                    key=|item| option_key(item)
                                    let:item
                                >
                                    <SearchSourceOptionRow item=item />
                                </For>
                            </ComboboxOptionGroup>
                        }
                    }
                </For>
                <Show when=move || grouped_options.get().is_empty()>
                    <ComboboxOption text="No results" value="__empty__" disabled=Signal::from(true)>
                        <Caption1>"No results"</Caption1>
                    </ComboboxOption>
                </Show>
            </Combobox>
        </div>
    }
}
