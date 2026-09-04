//! Not-found demo route.
#![allow(missing_docs)]

use leptos::prelude::*;
use uf_integrations::{ShellAuthMenu, UnifiedFieldNotFoundPage};

#[component]
pub fn NotFoundDemoPage() -> impl IntoView {
    view! {
        <UnifiedFieldNotFoundPage>
            <ShellAuthMenu slot:auth_menu>
                <span>"Demo user"</span>
            </ShellAuthMenu>
        </UnifiedFieldNotFoundPage>
    }
}
