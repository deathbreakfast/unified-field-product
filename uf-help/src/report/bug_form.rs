//! Bug report form fields.

use leptos::prelude::*;
use uf_product::primitives::{
    Field, Flex, FlexGap, Input, InputAppearance, Textarea, TextareaAppearance,
};

/// No-account bug form controls.
#[component]
pub fn BugFormFields(
    title: RwSignal<String>,
    description: RwSignal<String>,
    steps_to_repro: RwSignal<String>,
    expected: RwSignal<String>,
    actual: RwSignal<String>,
    app_version: RwSignal<String>,
    browser_os: RwSignal<String>,
    contact: RwSignal<String>,
) -> impl IntoView {
    view! {
        <Flex vertical=true gap=FlexGap::Medium>
            <Field label="Title" required=true>
                <Input bind=title appearance=InputAppearance::with_placeholder("Short summary") />
            </Field>
            <Field label="Description" required=true>
                <Textarea
                    bind=description
                    appearance=TextareaAppearance::with_placeholder("What went wrong?")
                />
            </Field>
            <Field label="Steps to reproduce" required=true>
                <Textarea
                    bind=steps_to_repro
                    appearance=TextareaAppearance::with_placeholder("1. …")
                />
            </Field>
            <Field label="Expected behavior" required=true>
                <Textarea bind=expected />
            </Field>
            <Field label="Actual behavior" required=true>
                <Textarea bind=actual />
            </Field>
            <Field label="App / version">
                <Input bind=app_version />
            </Field>
            <Field label="Browser / OS">
                <Input bind=browser_os />
            </Field>
            <Field label="Contact email (optional)">
                <Input bind=contact appearance=InputAppearance::with_placeholder("you@example.com") />
            </Field>
        </Flex>
    }
}
