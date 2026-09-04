//! Apps search launcher — Orbital [`Dialog`] on every viewport.
//!
//! A centered dialog fits a typeahead launcher better than a side/bottom drawer:
//! the panel is width-constrained and centered (`margin: auto`), and we avoid
//! Orbital bottom-drawer `align-items: flex-start` (children stay content-width
//! and hug the left edge of a full-bleed sheet).

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;
use uf_product::primitives::{
    Dialog, DialogBody, DialogContent, DialogSurface, DialogTitle, OpenBind,
};

use super::body::AppsLauncherBody;
use super::safe_route::safe_app_route_path;

/// Opens a searchable apps list. Host owns `open`.
///
/// Renders an Orbital [`Dialog`] (centered, max-width ~600px). The list stays
/// empty until the user types, then filters registered apps and navigates to
/// each hit's `route_path`.
#[component]
pub fn AppsLauncher(
    /// Host-owned open binding.
    #[prop(into)]
    open: RwSignal<bool>,
) -> impl IntoView {
    let query = RwSignal::new(String::new());
    let navigate = use_navigate();
    let open_bind: OpenBind = open.into();

    Effect::new(move |_| {
        if !open.get() {
            query.set(String::new());
        }
    });

    let on_select = Callback::new(move |route_path: String| {
        let Some(path) = safe_app_route_path(&route_path).map(str::to_string) else {
            return;
        };
        open.set(false);
        navigate(&path, NavigateOptions::default());
    });

    view! {
        <Dialog open=open_bind>
            <DialogSurface>
                // testid must live inside DialogSurface (teleported).
                <div data-testid="apps-launcher-dialog">
                    <DialogBody>
                        <DialogTitle>"Apps"</DialogTitle>
                        <DialogContent>
                            <AppsLauncherBody query=query on_select=on_select />
                        </DialogContent>
                    </DialogBody>
                </div>
            </DialogSurface>
        </Dialog>
    }
}
