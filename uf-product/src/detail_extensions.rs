//! Inventory contributions for optional product panels on a host detail page.
//!
//! An optional product offering (for example Magnon) submits a
//! [`DetailExtensionContribution`] to render a panel on another product's detail
//! page — Polaron org/team Overview today, uf-apps app detail once that host wires
//! it — without either host Cargo-depending on the contributing product. Host pages
//! render `<DetailExtensionSlot kind=.. scope_id=.. />`; when no contribution is
//! registered for that `kind` the slot renders nothing, so hosts stay valid whether
//! or not the contributing product is force-linked into the binary.

use leptos::prelude::*;

/// Which host surface a [`DetailExtensionContribution`] renders into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailExtensionKind {
    /// Polaron organization detail, Overview tab.
    PolaronOrgOverview,
    /// Polaron team detail, Overview tab.
    PolaronTeamOverview,
    /// uf-apps app detail, below `AppOverviewCard`.
    UfAppDetail,
}

/// One panel contributed by an optional product offering into a host detail page.
///
/// # Examples
///
/// ```rust,ignore
/// use leptos::prelude::*;
/// use uf_product::{DetailExtensionContribution, DetailExtensionKind};
///
/// fn render_org_panel(org_id: String) -> AnyView {
///     view! { <p>{format!("Strategy panel for org {org_id}")}</p> }.into_any()
/// }
///
/// inventory::submit! {
///     DetailExtensionContribution::new(
///         10,
///         "magnon_org_strategy_panel",
///         DetailExtensionKind::PolaronOrgOverview,
///         render_org_panel,
///     )
/// }
/// ```
pub struct DetailExtensionContribution {
    /// Sort key (lower first) among contributions sharing a [`DetailExtensionKind`].
    pub order: u8,
    /// Stable id for tests and docs.
    pub id: &'static str,
    /// Host surface this contribution renders into.
    pub kind: DetailExtensionKind,
    /// Render the panel for the given scope id (e.g. a Polaron org or team id).
    pub render: fn(scope_id: String) -> AnyView,
}

impl DetailExtensionContribution {
    /// Construct a contribution for inventory registration.
    #[must_use]
    pub const fn new(
        order: u8,
        id: &'static str,
        kind: DetailExtensionKind,
        render: fn(String) -> AnyView,
    ) -> Self {
        Self {
            order,
            id,
            kind,
            render,
        }
    }
}

inventory::collect!(DetailExtensionContribution);

/// No-op touch point so offering crates can force-link inventory into the binary.
pub fn register_detail_extensions() {}

/// Collect contributions for `kind`, sorted by [`DetailExtensionContribution::order`].
#[must_use]
pub fn collect_detail_extensions(
    kind: DetailExtensionKind,
) -> Vec<&'static DetailExtensionContribution> {
    let mut items: Vec<&'static DetailExtensionContribution> =
        inventory::iter::<DetailExtensionContribution>
            .into_iter()
            .filter(|c| c.kind == kind)
            .collect();
    items.sort_by_key(|c| c.order);
    items
}

/// Renders every registered [`DetailExtensionContribution`] for `kind`, in `order`.
///
/// Renders nothing when no contribution is registered for `kind` — hosts stay valid
/// whether or not the contributing product is force-linked into the binary.
///
/// # Examples
///
/// ```rust,ignore
/// use leptos::prelude::*;
/// use uf_product::{DetailExtensionKind, DetailExtensionSlot};
///
/// #[component]
/// fn OrgOverviewTab(org_id: String) -> impl IntoView {
///     view! {
///         <DetailExtensionSlot kind=DetailExtensionKind::PolaronOrgOverview scope_id=org_id />
///     }
/// }
/// ```
#[component]
pub fn DetailExtensionSlot(
    /// Host surface to render contributions for.
    kind: DetailExtensionKind,
    /// Id passed to each contribution's `render` (e.g. the Polaron org or team id).
    #[prop(into)]
    scope_id: String,
) -> impl IntoView {
    collect_detail_extensions(kind)
        .into_iter()
        .map(|c| (c.render)(scope_id.clone()))
        .collect_view()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render_probe(scope_id: String) -> AnyView {
        view! { <span data-testid="detail-extension-probe">{scope_id}</span> }.into_any()
    }

    inventory::submit! {
        DetailExtensionContribution::new(
            20,
            "detail_extensions_test_probe_org",
            DetailExtensionKind::PolaronOrgOverview,
            render_probe,
        )
    }

    inventory::submit! {
        DetailExtensionContribution::new(
            5,
            "detail_extensions_test_probe_app",
            DetailExtensionKind::UfAppDetail,
            render_probe,
        )
    }

    #[test]
    fn collect_detail_extensions_filters_by_kind_happy_path() {
        let org_items = collect_detail_extensions(DetailExtensionKind::PolaronOrgOverview);
        assert!(org_items
            .iter()
            .all(|c| c.kind == DetailExtensionKind::PolaronOrgOverview));
        assert!(org_items
            .iter()
            .any(|c| c.id == "detail_extensions_test_probe_org"));
        assert!(!org_items
            .iter()
            .any(|c| c.id == "detail_extensions_test_probe_app"));
    }

    #[test]
    fn collect_detail_extensions_sorts_by_order_happy_path() {
        let items = collect_detail_extensions(DetailExtensionKind::PolaronOrgOverview);
        let mut prev = 0u8;
        for item in &items {
            assert!(item.order >= prev, "contributions must sort by order");
            prev = item.order;
            assert!(!item.id.is_empty());
        }
    }

    #[test]
    fn collect_detail_extensions_empty_when_no_match_sad_path() {
        let items = collect_detail_extensions(DetailExtensionKind::PolaronTeamOverview);
        assert!(
            items
                .iter()
                .all(|c| c.id != "detail_extensions_test_probe_org"
                    && c.id != "detail_extensions_test_probe_app"),
            "PolaronTeamOverview has no test fixtures registered against it"
        );
    }

    // Rendering `AnyView` to HTML panics unless the `ssr` feature is enabled (see
    // `tachys::view::any_view`); `collect_view()` over an *empty* set of contributions
    // does not hit that path, so the empty-slot test above runs unconditionally, but
    // this one needs `--features ssr` (`cargo test -p uf-product --features ssr`).
    #[test]
    #[cfg_attr(
        not(feature = "ssr"),
        ignore = "renders AnyView to HTML; requires the `ssr` feature"
    )]
    fn detail_extension_slot_renders_matching_contribution_with_scope_id_happy_path() {
        let html = DetailExtensionSlot(DetailExtensionSlotProps {
            kind: DetailExtensionKind::PolaronOrgOverview,
            scope_id: "org-42".to_string(),
        })
        .to_html();
        assert!(
            html.contains("detail-extension-probe"),
            "expected the registered PolaronOrgOverview probe to render, got: {html}"
        );
        assert!(
            html.contains("org-42"),
            "expected scope_id to thread through to the rendered output, got: {html}"
        );
    }

    #[test]
    fn detail_extension_slot_ignores_other_kinds_sad_path() {
        let html = DetailExtensionSlot(DetailExtensionSlotProps {
            kind: DetailExtensionKind::PolaronTeamOverview,
            scope_id: "team-7".to_string(),
        })
        .to_html();
        assert!(
            !html.contains("detail-extension-probe"),
            "PolaronTeamOverview has no registered contribution and must render nothing, got: {html}"
        );
    }
}
