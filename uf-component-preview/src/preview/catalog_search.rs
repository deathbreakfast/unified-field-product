use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use orbital_base_components::{AutoCompleteSize, Handler};
use uf_product::components::{
    AutoComplete, AutoCompleteAppearance, AutoCompleteEvents, AutoCompleteOption, Body2, Caption1,
};

use super::collect_preview_registrations;
use super::navigation::{
    find_preview_registration_by_search_value, preview_href, preview_registration_search_value,
};

/// AppBar search field for jumping to any preview registration.
#[component]
pub fn PreviewCatalogSearch() -> impl IntoView {
    let value = RwSignal::new(String::new());
    let registrations = StoredValue::new(collect_preview_registrations());
    let navigate = use_navigate();
    let navigate_store = StoredValue::new(navigate);

    let on_select = Handler::from({
        move |search_value: String| {
            let Some(item) = find_preview_registration_by_search_value(
                &search_value,
                &registrations.get_value(),
            ) else {
                return;
            };
            let href = preview_href(item.slug);
            navigate_store.with_value(|navigate| {
                navigate(&href, Default::default());
            });
        }
    });

    view! {
        <div
            data-testid="preview-catalog-search"
            style="width: 100%; max-width: 320px;"
        >
            <AutoComplete
                bind=value
                appearance=AutoCompleteAppearance {
                    placeholder: "Search components…".into(),
                    size: Signal::from(AutoCompleteSize::Small),
                    clear_after_select: Signal::from(true),
                    blur_after_select: Signal::from(true),
                    ..Default::default()
                }
                events=AutoCompleteEvents {
                    on_select: Some(on_select),
                }
            >
                <For
                    each=move || registrations.get_value()
                    key=|item| item.slug
                    children=move |item| {
                        let search_value = preview_registration_search_value(item);
                        let label = item.label.to_string();
                        let detail = match (item.section.is_empty(), item.category.is_empty()) {
                            (false, false) => format!("{} · {}", item.section, item.category),
                            (false, true) => item.section.to_string(),
                            (true, false) => item.category.to_string(),
                            (true, true) => String::new(),
                        };
                        view! {
                            <AutoCompleteOption value=search_value>
                                <div style="display: flex; flex-direction: column; gap: 2px;">
                                    <Body2>{label}</Body2>
                                    {(!detail.is_empty()).then(|| view! { <Caption1>{detail.clone()}</Caption1> })}
                                </div>
                            </AutoCompleteOption>
                        }
                    }
                />
            </AutoComplete>
        </div>
    }
}
