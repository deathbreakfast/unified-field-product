//! Auth route stubs for product UI e2e (`/auth` → `/auth/signin`).
#![allow(missing_docs)]

use leptos::prelude::*;
use uf_product::components::{Body1, Title3};
use uf_product::primitives::{Flex, FlexGap};

/// Sign-in placeholder so `/auth/signin` is not the shell Not Found / Coming Soon chrome.
#[component]
pub fn AuthSignInPage() -> impl IntoView {
    view! {
        <main data-testid="auth-signin-page" style="padding: 24px; max-width: 720px;">
            <Flex vertical=true gap=FlexGap::Medium full_width=true>
                <Title3>"Sign in"</Title3>
                <Body1>"E2e auth sign-in route (host stub — no lepton Backend)."</Body1>
            </Flex>
        </main>
    }
}
