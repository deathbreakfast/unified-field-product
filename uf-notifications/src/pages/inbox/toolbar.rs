use leptos::prelude::*;

use uf_product::components::{SearchBox, SearchBoxAppearance, SearchBoxBind};
use uf_product::primitives::{Button, ButtonAppearance, Flex, FlexAlign, FlexGap, FlexWrap};

use crate::server::NotificationReadFilter;

/// Search box, read-status filter buttons, and a "Mark all read" action.
#[component]
pub fn NotificationToolbar(
    /// Two-way signal holding the search query.
    search_query: RwSignal<String>,
    /// Two-way signal holding the filter value to apply.
    filter: RwSignal<NotificationReadFilter>,
    /// Callback invoked when mark all read occurs.
    on_mark_all_read: impl Fn(leptos::ev::MouseEvent) + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <Flex gap=FlexGap::Medium align=FlexAlign::Center wrap=FlexWrap::Wrap full_width=true>
            <SearchBox
                bind=SearchBoxBind::from(search_query)
                appearance=SearchBoxAppearance::with_placeholder("Search notifications...")
            />
            <Flex gap=FlexGap::Small>
                <Button
                    appearance=ButtonAppearance::Subtle
                    on_click=Callback::new(move |_| filter.set(NotificationReadFilter::All))
                >
                    "All"
                </Button>
                <Button
                    appearance=ButtonAppearance::Subtle
                    on_click=Callback::new(move |_| filter.set(NotificationReadFilter::Unread))
                >
                    "Unread"
                </Button>
                <Button
                    appearance=ButtonAppearance::Subtle
                    on_click=Callback::new(move |_| filter.set(NotificationReadFilter::Read))
                >
                    "Read"
                </Button>
            </Flex>
            <Button
                appearance=ButtonAppearance::Secondary
                on_click=Callback::new(on_mark_all_read)
            >
                "Mark all read"
            </Button>
        </Flex>
    }
}
