//! Coming-soon demo route.
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
