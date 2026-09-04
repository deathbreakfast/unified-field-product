//! Feature request form fields.

use leptos::prelude::*;
use uf_product::primitives::{
    Field, Flex, FlexGap, Input, InputAppearance, Textarea, TextareaAppearance,
};

/// No-account feature request form controls.
#[component]
pub fn FeatureFormFields(
    title: RwSignal<String>,
    problem: RwSignal<String>,
    proposed: RwSignal<String>,
    alternatives: RwSignal<String>,
    contact: RwSignal<String>,
) -> impl IntoView {
    view! {
        <Flex vertical=true gap=FlexGap::Medium>
            <Field label="Title" required=true>
                <Input bind=title appearance=InputAppearance::with_placeholder("Short summary") />
            </Field>
            <Field label="Problem / use case" required=true>
                <Textarea
                    bind=problem
                    appearance=TextareaAppearance::with_placeholder("What are you trying to do?")
                />
            </Field>
            <Field label="Proposed solution" required=true>
                <Textarea bind=proposed />
            </Field>
            <Field label="Alternatives considered (optional)">
                <Textarea bind=alternatives />
            </Field>
            <Field label="Contact email (optional)">
                <Input bind=contact appearance=InputAppearance::with_placeholder("you@example.com") />
            </Field>
        </Flex>
    }
}
