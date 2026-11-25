//! [`PrincipalPicker`]: one typeahead for teams, groups and people.

use std::time::Duration;

use leptos::prelude::*;

use super::parts::{principal_persona, tint_badge};
use super::server::{search_access_principals, MIN_QUERY_CHARS};
use super::{AccessEntry, PrincipalKind, PrincipalRef, PrincipalResults, Role};
use crate::components::{Caption2, Tag, TagAppearance, TagSize};
use crate::primitives::{
    Combobox, ComboboxAppearance, ComboboxBind, ComboboxOption, ComboboxOptionGroup, Flex,
    FlexAlign, FlexGap, FlexJustify, SpacingHorizontal, SpacingInset, SpacingVertical,
};

/// Wait after the last keystroke before searching.
const DEBOUNCE: Duration = Duration::from_millis(200);

/// One pickable row: a principal, and the role it already holds on the card, if any.
#[derive(Clone, Debug, PartialEq)]
pub struct PickerRow {
    /// The principal.
    pub principal: PrincipalRef,
    /// Its role on the card already. Such rows are shown dimmed and can't be picked.
    pub existing: Option<Role>,
}

/// One heading in the open list.
#[derive(Clone, Debug, PartialEq)]
pub struct PickerGroup {
    /// Stable key: `teams`, `groups`, `people`.
    pub key: &'static str,
    /// Heading text.
    pub label: &'static str,
    /// Rows under the heading. Never empty.
    pub rows: Vec<PickerRow>,
}

/// What the open list shows: groups, or one status line when there are none.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PickerListing {
    /// Teams, Groups, People, in that order; empty groups are left out.
    pub groups: Vec<PickerGroup>,
    /// A hint, "Searching…" or no match.
    pub status: Option<&'static str>,
}

/// Build the open list for `query`. `results` is `None` while the search runs.
///
/// Groups come in the order Teams, Groups, People, and a group with no rows has no heading,
/// so a host with no managed team groups never shows Teams.
#[must_use]
pub fn build_listing(
    query: &str,
    results: Option<&PrincipalResults>,
    existing: &[AccessEntry],
) -> PickerListing {
    if query.trim().chars().count() < MIN_QUERY_CHARS {
        return PickerListing {
            groups: Vec::new(),
            status: Some("Type 2 or more letters to search."),
        };
    }
    let Some(results) = results else {
        return PickerListing {
            groups: Vec::new(),
            status: Some("Searching…"),
        };
    };
    let row = |principal: &PrincipalRef| PickerRow {
        existing: existing
            .iter()
            .find(|e| e.principal.key() == principal.key())
            .map(|e| e.role.clone()),
        principal: principal.clone(),
    };
    let groups: Vec<PickerGroup> = [
        ("teams", "Teams", &results.teams),
        ("groups", "Groups", &results.groups),
        ("people", "People", &results.people),
    ]
    .into_iter()
    .filter(|(_, _, rows)| !rows.is_empty())
    .map(|(key, label, rows)| PickerGroup {
        key,
        label,
        rows: rows.iter().map(row).collect(),
    })
    .collect();
    let status = groups.is_empty().then_some("No one matches.");
    PickerListing { groups, status }
}

/// Server answers, keyed by the query they answer.
type Answers = RwSignal<Option<(String, PrincipalResults)>>;

/// Searches 200 ms after `query` stops changing, once it has two characters. Stale answers
/// are dropped.
fn debounced_search(query: RwSignal<String>) -> (Answers, RwSignal<Option<String>>) {
    let results: Answers = RwSignal::new(None);
    let search_error = RwSignal::new(None::<String>);
    let generation = StoredValue::new(0_u64);
    Effect::new(move |_| {
        let q = query.get().trim().to_string();
        generation.update_value(|g| *g += 1);
        let current = generation.get_value();
        search_error.set(None);
        if q.chars().count() < MIN_QUERY_CHARS {
            results.set(None);
            return;
        }
        // The timer fires outside any owner; run the search under the picker's, so it stops
        // with the picker.
        let owner = Owner::current();
        set_timeout(
            move || {
                if generation.try_get_value() != Some(current) {
                    return;
                }
                let Some(owner) = owner else {
                    return;
                };
                owner.with(|| {
                    leptos::task::spawn_local_scoped(async move {
                        let outcome = search_access_principals(q.clone()).await;
                        if generation.try_get_value() != Some(current) {
                            return;
                        }
                        match outcome {
                            Ok(found) => {
                                results.try_set(Some((q, found)));
                            }
                            Err(e) => {
                                results.try_set(Some((q, PrincipalResults::default())));
                                search_error.try_set(Some(e.to_string()));
                            }
                        }
                    });
                });
            },
            DEBOUNCE,
        );
    });
    (results, search_error)
}

fn status_option(text: &'static str) -> impl IntoView {
    view! {
        <ComboboxOption text=text value="__status__" disabled=Signal::from(true)>
            <span data-testid="principal-picker-status">
                <Caption2>{text}</Caption2>
            </span>
        </ComboboxOption>
    }
}

fn row_view(row: PickerRow) -> impl IntoView {
    let key = row.principal.key();
    let disabled = row.existing.is_some();
    let team = matches!(row.principal.kind, PrincipalKind::Team { .. })
        .then(|| tint_badge("Team", "principal-picker-team-badge"));
    let existing = row.existing.map(|role| {
        view! {
            <span data-testid="principal-picker-existing-role">
                <Caption2>{role.label()}</Caption2>
            </span>
        }
    });
    let persona = principal_persona(&row.principal);
    // The vertical inset keeps every row a 44 px touch target on a phone.
    view! {
        <ComboboxOption
            text=row.principal.label.clone()
            value=key.clone()
            disabled=Signal::from(disabled)
        >
            <Flex
                justify=FlexJustify::SpaceBetween
                align=FlexAlign::Center
                gap=FlexGap::Small
                full_width=true
                padding=SpacingInset::symmetric(SpacingHorizontal::None, SpacingVertical::XS)
                attr:data-testid="principal-picker-row"
                attr:data-key=key
                attr:data-disabled=disabled.to_string()
            >
                <Flex gap=FlexGap::Small align=FlexAlign::Center>
                    {persona}
                    {team}
                </Flex>
                {existing}
            </Flex>
        </ComboboxOption>
    }
}

/// The open list's groups, then its status line.
pub fn listing_view(listing: PickerListing) -> impl IntoView {
    let groups = listing
        .groups
        .into_iter()
        .map(|group| {
            let rows = group.rows.into_iter().map(row_view).collect_view();
            view! {
                <ComboboxOptionGroup
                    label=group.label
                    attr:data-testid="principal-picker-group"
                    attr:data-group=group.key
                >
                    {rows}
                </ComboboxOptionGroup>
            }
        })
        .collect_view();
    let status = listing.status.map(status_option);
    view! { {groups} {status} }
}

/// One typeahead for teams, groups and people, for giving access to a resource.
///
/// Rows are Orbital `Persona`s grouped Teams (team groups, with their size), Groups and
/// People (service users show their owner). A group with no matches has no heading, so a
/// host with no managed team groups shows no Teams. Rows already on the card show dimmed
/// with their role. The search starts after two characters, 200 ms after the last
/// keystroke, through the host's [`super::ResourceAccessBackend::search_principals`]. The
/// pick shows as a tag with a remove button.
///
/// ```rust,ignore
/// use leptos::prelude::*;
/// use uf_product::access::{PrincipalPicker, PrincipalRef};
///
/// let who = RwSignal::new(None::<PrincipalRef>);
/// view! { <PrincipalPicker value=who /> };
/// ```
#[component]
pub fn PrincipalPicker(
    /// The picked principal.
    value: RwSignal<Option<PrincipalRef>>,
    /// Rows already on the card; they show dimmed with their role.
    #[prop(optional, into)]
    existing: Signal<Vec<AccessEntry>>,
    /// Disables the input and the tag's remove button.
    #[prop(optional, into)]
    disabled: Signal<bool>,
) -> impl IntoView {
    let query = RwSignal::new(String::new());
    let selected = RwSignal::new(Vec::<String>::new());
    // Rows the list has offered, so a pick resolves after the list has moved on.
    let shown = StoredValue::new(Vec::<PrincipalRef>::new());
    let (results, search_error) = debounced_search(query);

    let listing = Memo::new(move |_| {
        let q = query.get();
        let answered = results.get();
        let answered = answered
            .as_ref()
            .filter(|(asked, _)| asked == q.trim())
            .map(|(_, found)| found);
        existing.with(|existing| build_listing(&q, answered, existing))
    });

    Effect::new(move |_| {
        let Some(key) = selected.get().last().cloned() else {
            return;
        };
        selected.set(Vec::new());
        query.set(String::new());
        if let Some(pick) = shown.with_value(|rows| rows.iter().find(|p| p.key() == key).cloned()) {
            value.set(Some(pick));
        }
    });

    let options = move || {
        let listing = listing.get();
        shown.update_value(|rows| {
            for group in &listing.groups {
                for row in &group.rows {
                    if !rows.iter().any(|p| p.key() == row.principal.key()) {
                        rows.push(row.principal.clone());
                    }
                }
            }
        });
        listing_view(listing)
    };
    let appearance = move || ComboboxAppearance {
        placeholder: MaybeProp::from("Search teams, groups or people".to_string()),
        disabled,
        clearable: false,
        disable_filter: true,
        ..Default::default()
    };

    view! {
        <Flex vertical=true gap=FlexGap::Small full_width=true attr:data-testid="principal-picker">
            {move || match value.get() {
                Some(picked) => {
                    let label = picked.label.clone();
                    let key = picked.key();
                    view! {
                        <span data-testid="principal-picker-pick" data-key=key>
                            <Tag
                                size=TagSize::Medium
                                appearance=TagAppearance::Outline
                                dismissible=Signal::derive(move || !disabled.get())
                                dismiss_label=format!("Remove {label}")
                                on_dismiss=move |_: leptos::ev::MouseEvent| value.set(None)
                            >
                                {picked.label.clone()}
                            </Tag>
                        </span>
                    }
                    .into_any()
                }
                None => view! {
                    <Combobox bind=ComboboxBind::new(query, selected) appearance=appearance()>
                        {options}
                    </Combobox>
                }
                .into_any(),
            }}
            {move || search_error.get().map(|msg| view! {
                <span data-testid="principal-picker-error" role="alert">
                    <Caption2>{msg}</Caption2>
                </span>
            })}
        </Flex>
    }
}
