//! Auth-gate demo pages that exercise [`RequireAuthenticated`] directly.
#![allow(missing_docs)]

use leptos::prelude::*;
use uf_product::components::{Body1, Title3};
use uf_product::primitives::{Flex, FlexGap};
use uf_product::routes::RequireAuthenticated;

#[component]
pub fn GateEmailPage() -> impl IntoView {
    view! {
        <RequireAuthenticated requires_email_verification=true>
            <main data-testid="gate-email-content" style="padding: 24px; max-width: 720px;">
                <Flex vertical=true gap=FlexGap::Medium full_width=true>
                    <Title3>"Email verified content"</Title3>
                    <Body1>"Visible only when authenticated and email_verified."</Body1>
                </Flex>
            </main>
        </RequireAuthenticated>
    }
}

#[component]
pub fn GatePermissionPage() -> impl IntoView {
    view! {
        <RequireAuthenticated permission_name="e2e.permission.deny">
            <main data-testid="gate-permission-content" style="padding: 24px; max-width: 720px;">
                <Flex vertical=true gap=FlexGap::Medium full_width=true>
                    <Title3>"Permission content"</Title3>
                    <Body1>"Should not appear while the deny permission is checked."</Body1>
                </Flex>
            </main>
        </RequireAuthenticated>
    }
}

#[component]
pub fn GatePermissionAllowPage() -> impl IntoView {
    view! {
        <RequireAuthenticated permission_name="e2e.permission.allow">
            <main data-testid="gate-permission-allow-content" style="padding: 24px; max-width: 720px;">
                <Flex vertical=true gap=FlexGap::Medium full_width=true>
                    <Title3>"Permission allow content"</Title3>
                    <Body1>"Visible when harness seeds permission_allow for e2e.permission.allow."</Body1>
                </Flex>
            </main>
        </RequireAuthenticated>
    }
}
