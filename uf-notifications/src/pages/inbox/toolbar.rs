use leptos::prelude::*;

use uf_product::components::{SearchBox, SearchBoxAppearance, SearchBoxBind};
use uf_product::primitives::{Flex, FlexAlign, FlexGap, FlexWrap, Select, SelectBind};

use crate::server::get_inbox_apps;

/// Search box and an app filter (`""` means all apps).
#[component]
pub fn NotificationToolbar(
    /// Two-way signal holding the search query.
    search_query: RwSignal<String>,
    /// Two-way signal holding the selected app id; empty for all apps.
    app: RwSignal<String>,
    /// `true` once hydrated; the app options load only then, since reading a
    /// LocalResource during SSR outside a Suspense panics.
    hydrated: Signal<bool>,
) -> impl IntoView {
    // Client-only, like the tab counts: the options fill in after hydration.
    let apps = LocalResource::new(get_inbox_apps);

    view! {
        <Flex gap=FlexGap::Small align=FlexAlign::Center wrap=FlexWrap::Wrap full_width=true>
            <SearchBox
                bind=SearchBoxBind::from(search_query)
                appearance=SearchBoxAppearance::with_placeholder("Search notifications")
            />
            // Orbital's Select takes no aria-label; a visually hidden label (the
            // theme ships `sr-only`) names it for screen readers.
            <label for="notifications-app-filter" class="sr-only">"Filter by app"</label>
            <Select bind=SelectBind {
                id: "notifications-app-filter".to_string().into(),
                ..SelectBind::new(app)
            }>
                <option value="">"All apps"</option>
                {move || {
                    hydrated
                        .get()
                        .then(|| apps.get())
                        .flatten()
                        .and_then(Result::ok)
                        .unwrap_or_default()
                        .into_iter()
                        .map(|a| view! { <option value=a.id>{a.name}</option> })
                        .collect_view()
                }}
            </Select>
        </Flex>
    }
}
