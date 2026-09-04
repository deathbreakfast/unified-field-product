//! Coming-soon demo route (full-page shell from uf-integrations).
#![allow(missing_docs)]

use leptos::prelude::*;
use uf_integrations::{ShellAuthMenu, UnifiedFieldComingSoonPage};

#[component]
pub fn ComingSoonDemoPage() -> impl IntoView {
    view! {
        <UnifiedFieldComingSoonPage>
            <ShellAuthMenu slot:auth_menu>
                <span>"Demo user"</span>
            </ShellAuthMenu>
        </UnifiedFieldComingSoonPage>
    }
}
