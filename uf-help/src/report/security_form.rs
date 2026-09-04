//! Security report form fields.

use leptos::prelude::*;
use uf_product::primitives::{
    Field, Flex, FlexGap, Input, InputAppearance, Select, Textarea, TextareaAppearance,
};

/// No-account security report form controls (private channel only).
#[component]
pub fn SecurityFormFields(
    summary: RwSignal<String>,
    description: RwSignal<String>,
    repro: RwSignal<String>,
    affected: RwSignal<String>,
    severity: RwSignal<String>,
    contact: RwSignal<String>,
) -> impl IntoView {
    view! {
        <Flex vertical=true gap=FlexGap::Medium>
            <p>
                "This form goes through the private advisory channel. Do not paste secrets you cannot rotate."
            </p>
            <Field label="Summary" required=true>
                <Input bind=summary appearance=InputAppearance::with_placeholder("Short summary") />
            </Field>
            <Field label="Description and impact" required=true>
                <Textarea bind=description />
            </Field>
            <Field label="Steps to reproduce / PoC" required=true>
                <Textarea
                    bind=repro
                    appearance=TextareaAppearance::with_placeholder("Repro steps or PoC outline")
                />
            </Field>
            <Field label="Affected components / versions" required=true>
                <Textarea bind=affected />
            </Field>
            <Field label="Severity (optional)">
                <Select bind=severity>
                    <option value="">"Select severity"</option>
                    <option value="low">"low"</option>
                    <option value="medium">"medium"</option>
                    <option value="high">"high"</option>
                    <option value="critical">"critical"</option>
                </Select>
            </Field>
            <Field label="Contact email (strongly urged)">
                <Input bind=contact appearance=InputAppearance::with_placeholder("you@example.com") />
            </Field>
        </Flex>
    }
}
