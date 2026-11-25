//! Build-version fixture: a server function that fails the way a stale build does.
#![allow(missing_docs)]

use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::A;
use uf_product::components::{Body1, Title3};
use uf_product::primitives::{Button, Flex, FlexGap};
use uf_product::services::permission_server_errors::report_server_fn_error;

/// Always fails with the deserialisation error an old client sees after a deploy.
#[server(E2eBuildVersionStaleCall)]
pub async fn e2e_build_version_stale_call() -> Result<String, ServerFnError> {
    Err(ServerFnError::Deserialization(
        "missing field `id` at line 1 column 2".into(),
    ))
}

/// `/build-version-fixture`: "Load data" calls the failing server function and
/// reports the error the way product pages do; the link makes a client-side
/// route change.
#[component]
pub fn BuildVersionFixturePage() -> impl IntoView {
    let failed = RwSignal::new(false);
    let load = Callback::new(move |_| {
        spawn_local(async move {
            if let Err(err) = e2e_build_version_stale_call().await {
                report_server_fn_error(&err);
                failed.set(true);
            }
        });
    });

    view! {
        <main data-testid="build-version-fixture" style="padding: 24px; max-width: 720px;">
            <Flex vertical=true gap=FlexGap::Medium full_width=true>
                <Title3>"Build version fixture"</Title3>
                <Button attr:data-testid="build-version-fixture-load" on_click=load>
                    "Load data"
                </Button>
                <span data-testid="build-version-fixture-nav">
                    <A href="/workspace-search-hit">"Go to another page"</A>
                </span>
                <Show when=move || failed.get()>
                    <Body1 attr:data-testid="build-version-fixture-failed">"Load failed"</Body1>
                </Show>
            </Flex>
        </main>
    }
}
