use leptos::prelude::*;
use uf_product::primitives::{BadgeColor, Button, ButtonAppearance, CounterBadge};

#[component]
pub fn NotificationBellTrigger(
    /// Resource that loads the count data.
    count_res: Resource<Result<usize, ServerFnError>>,
) -> impl IntoView {
    view! {
        <div data-testid="notification-bell-container">
            <div id="notification-bell" data-testid="notification-bell">
                <Transition fallback=move || {
                    view! {
                        <Button
                            appearance=ButtonAppearance::Subtle
                            icon=icondata::AiBellOutlined
                            attr:aria-label="Notifications"
                        />
                    }
                }>
                    {move || {
                        let count = count_res.get().and_then(Result::ok).unwrap_or(0) as u32;
                        if count > 0 {
                            view! {
                                <CounterBadge
                                    count=count
                                    color=Signal::from(BadgeColor::Danger)
                                >
                                    <Button
                                        appearance=ButtonAppearance::Subtle
                                        icon=icondata::AiBellOutlined
                                        attr:aria-label="Notifications"
                                    />
                                </CounterBadge>
                            }
                            .into_any()
                        } else {
                            view! {
                                <Button
                                    appearance=ButtonAppearance::Subtle
                                    icon=icondata::AiBellOutlined
                                    attr:aria-label="Notifications"
                                />
                            }
                            .into_any()
                        }
                    }}
                </Transition>
            </div>
        </div>
    }
}
