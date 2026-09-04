use crate::server::NotificationDto;
use leptos::html::Div;
use leptos::prelude::*;
use orbital_core_components::ScrollArea;
use uf_product::components::Caption2;
use uf_product::primitives::{Flex, FlexGap, MenuItem};

use super::item::NotificationBellItem;

#[component]
pub fn NotificationBellDropdown(
    /// Two-way signal holding the list of items to render.
    items: RwSignal<Vec<NotificationDto>>,
    /// Two-way signal controlling whether ever loaded is enabled.
    ever_loaded: RwSignal<bool>,
    /// Two-way signal controlling whether it is loading more.
    is_loading_more: RwSignal<bool>,
    /// Two-way signal controlling whether there is more.
    has_more: RwSignal<bool>,
    /// DOM node reference for the scroll element.
    scroll_el: NodeRef<Div>,
    /// Two-way signal holding the trigger element/state.
    trigger: RwSignal<u64>,
) -> impl IntoView {
    let (style_sheet, class_names) = turf::inline_style_sheet_values! {
        .EmptyState {
            color: var(--orb-color-text-tertiary);
        }
    };

    view! {
        <style>{style_sheet}</style>
        <MenuItem value="header" disabled=true>
            <Caption2>"Notifications"</Caption2>
        </MenuItem>
        // ScrollArea documents `style` for bounded max-height (Orbital API, not a fork).
        <ScrollArea
            node_ref=scroll_el
            style="display: block; max-height: 360px; width: 100%; box-sizing: border-box;"
        >
            {move || {
                let current_items = items.get();
                let loaded = ever_loaded.get();

                if current_items.is_empty() {
                    if loaded {
                        view! {
                            <div class="orbital-menu-item orbital-menu-item--disabled">
                                <span style="flex-grow: 1">
                                    <Caption2 class=class_names.empty_state>"No unread notifications."</Caption2>
                                </span>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="orbital-menu-item orbital-menu-item--disabled">
                                <span style="flex-grow: 1">
                                    <Caption2 class=class_names.empty_state>"Loading..."</Caption2>
                                </span>
                            </div>
                        }.into_any()
                    }
                } else {
                    view! {
                        <Flex vertical=true gap=FlexGap::Size(4) full_width=true>
                            {current_items.into_iter().map(|item| {
                                view! { <NotificationBellItem notification=item items=items trigger=trigger /> }
                            }).collect_view()}
                        </Flex>
                    }.into_any()
                }
            }}
            {move || {
                if is_loading_more.get() && has_more.get() && ever_loaded.get() {
                    view! {
                        <div class="orbital-menu-item orbital-menu-item--disabled">
                            <span style="flex-grow: 1">
                                <Caption2 class=class_names.empty_state>"Loading more..."</Caption2>
                            </span>
                        </div>
                    }.into_any()
                } else {
                    let _: () = view! { <></> };
                    ().into_any()
                }
            }}
        </ScrollArea>
        <MenuItem value="view_all">"View all"</MenuItem>
    }
}
