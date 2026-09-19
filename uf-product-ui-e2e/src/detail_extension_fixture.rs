//! Test-only `DetailExtensionKind::UfAppDetail` contribution.
//!
//! Proves the wiring Magnon DESIGN.md M0b task P2 requires: `uf-apps`'s
//! `AppDetailPage` actually mounts `<DetailExtensionSlot kind=UfAppDetail
//! scope_id=.. />` below `AppOverviewCard`, and threads the real app slug
//! through as `scope_id`. Registered unconditionally via `inventory::submit!`
//! at crate load (no separate force-link call needed), so every app detail
//! page in this e2e host renders this probe — Playwright asserts its
//! `scope_id` text per app slug. This module never ships in a product host.

use leptos::prelude::*;
use uf_product::{DetailExtensionContribution, DetailExtensionKind};

fn render_probe(scope_id: String) -> AnyView {
    view! {
        <div data-testid="uf-app-detail-extension-probe">{scope_id}</div>
    }
    .into_any()
}

inventory::submit! {
    DetailExtensionContribution::new(
        10,
        "e2e_uf_app_detail_probe",
        DetailExtensionKind::UfAppDetail,
        render_probe,
    )
}
