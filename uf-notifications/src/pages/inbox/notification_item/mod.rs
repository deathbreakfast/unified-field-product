//! One inbox row, plus the pieces the bell shares with it: the app icon, the
//! "{sender} · {app} · {kind} · {time}" caption, day groups and the toast a
//! failed read change raises.

mod helpers;
mod time;

use std::collections::HashMap;

use leptos::prelude::*;
use leptos::task::spawn_local_scoped;
use leptos_router::NavigateOptions;
use orbital_core_components::{ListItem, ListItemTrailing, ToastOptions};
use uf_product::components::{Body1, Body1Strong, Caption1};
use uf_product::primitives::{
    Avatar, AvatarConfig, AvatarShape, Badge, BadgeSize, Button, ButtonAppearance, ButtonSize,
    Divider, Flex, FlexAlign, FlexGap, Menu, MenuItem, MenuPosition, MenuTrigger, ToastIntent,
    ToasterInjection,
};
use uuid::Uuid;

use crate::safe_url::safe_notification_nav_url;
use crate::server::{
    get_inbox_rows_page, mark_all_notifications_read, mark_notification_read,
    mark_notification_unread, NotificationInboxFilter, NotificationReadFilter, NotificationRow,
    Page, PageRequest,
};

pub(crate) use helpers::{item_style_sheet, mark_read_label};
pub(crate) use time::{row_day_group, use_local_time, DayGroup, NotificationTime};

use super::PREFERENCES_PATH;

/// "{sender} · {app} · {kind} · {time}", skipping the parts a row doesn't have.
#[component]
pub(crate) fn NotificationMeta(
    /// Row to describe.
    row: NotificationRow,
    /// Whether to render times in the reader's local zone.
    local: Signal<bool>,
) -> impl IntoView {
    let rest: String = [row.app_name.clone(), row.notification.kind_label.clone()]
        .into_iter()
        .flatten()
        .flat_map(|part| [" · ".to_string(), part])
        .collect();
    view! {
        <Caption1 attr:data-testid="notification-meta">
            <span data-testid="notification-sender">{row.sender_name}</span>
            {rest}
            " · "
            <NotificationTime
                created_at=row.created_at
                fallback=row.notification.created_at
                local=local
            />
        </Caption1>
    }
}

/// The sending app's icon: a square avatar from the app's name.
#[component]
pub(crate) fn AppIcon(
    /// The app's display name; unknown apps get a generic icon.
    app_name: Option<String>,
) -> impl IntoView {
    let name = app_name.unwrap_or_else(|| "Notifications".to_string());
    view! {
        <Avatar config=AvatarConfig {
            size: Some(32),
            ..AvatarConfig::shaped(name, AvatarShape::Square)
        } />
    }
}

/// Raise an error toast with Try again on `toaster` (read it with `use_context`
/// before any `.await`). Hosts without a toaster only log.
pub(crate) fn show_failure_toast(
    toaster: Option<ToasterInjection>,
    heading: String,
    body: &str,
    retry: Callback<()>,
) {
    let Some(toaster) = toaster else {
        return;
    };
    toaster.dispatch(
        ToastOptions::new(heading)
            .body(body)
            .intent(ToastIntent::Error.into())
            .footer_action_callback("Try again", true, Some(retry)),
    );
}

/// Bumped after a read change succeeds so the inbox tab counts refetch.
/// Provided by the inbox page; absent in the bell.
#[derive(Clone, Copy)]
pub(crate) struct CountsRefresh(pub RwSignal<u64>);

/// Context a read change needs, read once while the component is built
/// (event handlers and async tasks may run without a reactive owner).
#[derive(Clone, Copy)]
pub(crate) struct ReadChangeContext {
    /// Shell toaster for failure toasts; `None` on hosts without one.
    pub toaster: Option<ToasterInjection>,
    /// Inbox tab counts to refresh on success.
    pub counts: Option<CountsRefresh>,
}

impl ReadChangeContext {
    /// Read the toaster and counts refresh from context.
    pub(crate) fn from_context() -> Self {
        Self {
            toaster: use_context::<ToasterInjection>(),
            counts: use_context::<CountsRefresh>(),
        }
    }
}

/// One loaded slot of a notification list: `Some(row)`, or `None` standing in
/// for a page that failed to load. Orbital's paging hook drops fetch errors, so
/// the failure travels as data instead (and survives SSR to hydrate).
pub(crate) type LoadedRow = Option<NotificationRow>;

/// Fetch a page of rows for the bell or inbox, turning a failure into a single
/// `None` slot with nothing more to load.
pub(crate) async fn fetch_rows(
    request: PageRequest,
    query: Option<String>,
    read_filter: NotificationReadFilter,
    filter: NotificationInboxFilter,
) -> Result<Page<LoadedRow>, ServerFnError> {
    match get_inbox_rows_page(request, query, read_filter, filter).await {
        Ok(page) => Ok(Page {
            items: page.items.into_iter().map(Some).collect(),
            has_more: page.has_more,
            total_count: page.total_count,
            next_request_offset: page.next_request_offset,
            next_cursor: page.next_cursor,
        }),
        Err(e) => {
            leptos::logging::warn!("Failed to load notifications: {e}");
            Ok(Page {
                items: vec![None],
                has_more: false,
                total_count: Some(0),
                ..Page::default()
            })
        }
    }
}

/// Whether a loaded list hit a failed page.
pub(crate) fn has_failed_page(items: &[LoadedRow]) -> bool {
    items.iter().any(Option::is_none)
}

/// Mark every notification read; `on_done` runs after the server confirms.
/// On failure nothing changes and a toast offers Try again.
pub(crate) fn mark_all_read(toaster: Option<ToasterInjection>, on_done: Callback<()>) {
    spawn_local_scoped(async move {
        match mark_all_notifications_read().await {
            Ok(count) => {
                leptos::logging::log!(
                    "[notifications] mark_all_notifications_read: marked {count}"
                );
                on_done.run(());
            }
            Err(e) => {
                leptos::logging::warn!("Failed to mark all notifications read: {e}");
                show_failure_toast(
                    toaster,
                    "Couldn't mark all as read".to_string(),
                    "Nothing changed.",
                    Callback::new(move |()| mark_all_read(toaster, on_done)),
                );
            }
        }
    });
}

/// Set one notification read or unread, showing the change at once. When the
/// server refuses, the row flips back and a toast names the notification.
pub(crate) fn set_read_state(
    id: Uuid,
    title: String,
    is_read: bool,
    read_overrides: RwSignal<HashMap<Uuid, bool>>,
    ctx: ReadChangeContext,
) {
    read_overrides.update(|overrides| {
        overrides.insert(id, is_read);
    });
    spawn_local_scoped(async move {
        let result = if is_read {
            mark_notification_read(id).await
        } else {
            mark_notification_unread(id).await
        };
        if let Err(e) = result {
            leptos::logging::warn!("Failed to change notification read state: {e}");
            let _ = read_overrides.try_update(|overrides| {
                overrides.insert(id, !is_read);
            });
            let (heading, body) = if is_read {
                (
                    format!("Couldn't mark “{title}” as read"),
                    "It's still unread.",
                )
            } else {
                (
                    format!("Couldn't mark “{title}” as unread"),
                    "It's still marked read.",
                )
            };
            let retry = Callback::new(move |()| {
                set_read_state(id, title.clone(), is_read, read_overrides, ctx);
            });
            show_failure_toast(ctx.toaster, heading, body, retry);
        } else if let Some(CountsRefresh(counts)) = ctx.counts {
            let _ = counts.try_update(|n| *n += 1);
        }
    });
}

/// A single notification row in the inbox list.
///
/// Read state comes from `read_overrides` so the row updates in place when
/// toggled (compatible with keyed `<For>`). Mark read/unread, Open and the
/// ⋯ menu sit at the right edge, so Open and the menu line up on every row;
/// below `Md` only the menu stays and tapping the row opens it.
#[component]
#[allow(clippy::needless_pass_by_value)] // Leptos `#[component]` props must be owned.
pub fn NotificationItem(
    /// Notification to render, with sender, time and app.
    row: NotificationRow,
    /// Two-way signal holding the read overrides.
    read_overrides: RwSignal<HashMap<Uuid, bool>>,
    /// Navigation callback used to change routes.
    navigate: impl Fn(&str, NavigateOptions) + Clone + Send + Sync + 'static,
    /// `true` below `Md`: only the ⋯ menu shows.
    compact: Signal<bool>,
    /// Whether to render times in the reader's local zone.
    local: Signal<bool>,
) -> impl IntoView {
    let notification = row.notification.clone();
    let notification_id = notification.notification_id;
    let original_is_read = notification.is_read;
    let url = notification.url.clone();
    let title = StoredValue::new(notification.title.clone());
    let message = notification.message.clone();
    let kind = notification
        .kind_id
        .clone()
        .zip(notification.kind_label.clone());

    let is_read = Memo::new(move |_| {
        read_overrides
            .get()
            .get(&notification_id)
            .copied()
            .unwrap_or(original_is_read)
    });

    let (style_sheet, class_names) = item_style_sheet();
    let dot_base = class_names.unread_dot.clone();
    let dot_hidden_cls = class_names.unread_dot_hidden.clone();
    let dot_class = Signal::derive(move || {
        if is_read.get() {
            format!("{dot_base} {dot_hidden_cls}")
        } else {
            dot_base.clone()
        }
    });

    let ctx = ReadChangeContext::from_context();
    let toggle_read = Callback::new(move |()| {
        set_read_state(
            notification_id,
            title.get_value(),
            !is_read.get_untracked(),
            read_overrides,
            ctx,
        );
    });

    let open_navigate = navigate.clone();
    let open = Callback::new(move |()| {
        if !is_read.get_untracked() {
            set_read_state(
                notification_id,
                title.get_value(),
                true,
                read_overrides,
                ctx,
            );
        }
        if url.is_some() {
            open_navigate(
                safe_notification_nav_url(url.as_deref()),
                NavigateOptions::default(),
            );
        }
    });

    let kind_label = StoredValue::new(kind.as_ref().map(|(_, label)| label.clone()));
    let preferences_href = kind
        .as_ref()
        .map(|(id, _)| format!("{PREFERENCES_PATH}#kind-{id}"));
    let on_menu_select = Callback::new(move |key: String| match key.as_str() {
        "toggle-read" => toggle_read.run(()),
        "change-kind" => {
            if let Some(href) = &preferences_href {
                navigate(href, NavigateOptions::default());
            }
        }
        _ => {}
    });

    view! {
        <style>{style_sheet}</style>
        <ListItem
            on_click=open
            attr:data-testid="notification-item"
            attr:data-read=move || if is_read.get() { "true" } else { "false" }
        >
            <Flex align=FlexAlign::Start gap=FlexGap::Medium full_width=true>
                <AppIcon app_name=row.app_name.clone() />
                <Flex vertical=true gap=FlexGap::Size(4) class=class_names.content>
                    <Flex align=FlexAlign::Center gap=FlexGap::Small>
                        <Badge size=Signal::from(BadgeSize::ExtraSmall) class=dot_class />
                        {move || {
                            let title = title.get_value();
                            if is_read.get() {
                                view! { <Body1 attr:data-testid="notification-title">{title}</Body1> }
                                    .into_any()
                            } else {
                                view! {
                                    <Body1Strong attr:data-testid="notification-title">{title}</Body1Strong>
                                }
                                .into_any()
                            }
                        }}
                    </Flex>
                    <Body1 block=true class=class_names.message_text>{message}</Body1>
                    <NotificationMeta row=row local=local />
                </Flex>
            </Flex>
            <ListItemTrailing slot>
                <Flex
                    align=FlexAlign::Center
                    gap=FlexGap::Small
                    attr:data-testid="notification-actions"
                >
                    <Show when=move || !compact.get()>
                        <Button
                            size=ButtonSize::Small
                            appearance=ButtonAppearance::Subtle
                            on_click=Callback::new(move |_| toggle_read.run(()))
                        >
                            {move || mark_read_label(is_read.get())}
                        </Button>
                        <Button
                            size=ButtonSize::Small
                            appearance=ButtonAppearance::Secondary
                            on_click=Callback::new(move |_| open.run(()))
                            attr:data-testid="notification-open"
                        >
                            "Open"
                        </Button>
                    </Show>
                    <RowMenu kind_label=kind_label.get_value() is_read=is_read on_select=on_menu_select />
                </Flex>
            </ListItemTrailing>
        </ListItem>
    }
}

/// The ⋯ menu on an inbox row.
#[component]
fn RowMenu(
    /// The kind's label; `None` hides "Change how I get …".
    kind_label: Option<String>,
    /// Current read state, for the Mark read / Mark unread item.
    is_read: Memo<bool>,
    /// Runs with the chosen item's key.
    on_select: Callback<String>,
) -> impl IntoView {
    view! {
        <Menu
            on_select=move |key: &str| on_select.run(key.to_string())
            position=MenuPosition::BottomEnd
        >
            <MenuTrigger slot>
                <Button
                    size=ButtonSize::Small
                    appearance=ButtonAppearance::Subtle
                    icon=icondata::AiEllipsisOutlined
                    attr:aria-label="More actions"
                    attr:data-testid="notification-more"
                />
            </MenuTrigger>
            <MenuItem value="toggle-read">{move || mark_read_label(is_read.get())}</MenuItem>
            {kind_label.map(|label| {
                view! {
                    <Divider />
                    <MenuItem value="change-kind">{format!("Change how I get “{label}”")}</MenuItem>
                }
            })}
        </Menu>
    }
}
