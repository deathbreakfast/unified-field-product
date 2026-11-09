//! Floating actions fixture item for `/` (shell with `ShellFloatingActions`).

use leptos::prelude::*;
use uf_product::primitives::{FloatingActionsMenuItem, ToasterInjection};
use uf_product::FloatingActionContribution;

/// Toast text the fixture item posts when selected.
pub const FAM_FIXTURE_TOAST: &str = "E2E fixture action ran";

fn render_fixture_item() -> AnyView {
    let toaster = ToasterInjection::expect_context();
    view! {
        <FloatingActionsMenuItem
            tooltip="E2E fixture action".to_string()
            icon=icondata::AiExperimentOutlined
            testid="fam-e2e-fixture"
            on_click=Callback::new(move |_| {
                toaster.show(FAM_FIXTURE_TOAST);
            })
        />
    }
    .into_any()
}

inventory::submit! {
    FloatingActionContribution::new(10, "e2e-fixture", render_fixture_item)
}

/// Force-link the fixture's inventory row.
pub fn ensure_fam_fixture_linked() {
    uf_product::register_floating_action();
}
