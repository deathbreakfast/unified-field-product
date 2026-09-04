use leptos::prelude::*;
use uf_product::components::{Body1, Caption1, Card, SpacingSize};
use uf_product::primitives::Flex;

use super::page::{ChapterHeading, SectionHeading, TocClassNames};

#[component]
pub(super) fn PrinciplesChapter(
    /// Additional CSS class(es) to apply.
    classes: TocClassNames,
) -> impl IntoView {
    view! {
        <Card>
            <Flex
                vertical=true
                gap=SpacingSize::Size200.flex_gap()
                padding=SpacingSize::Size320.inset()
            >
                <ChapterHeading id="principles" title="Principles" class=classes.chapter />

                <Flex vertical=true gap=SpacingSize::Size120.flex_gap()>
                    <SectionHeading
                        id="principles-use"
                        title="How to use these principles"
                        class=classes.section
                    />
                    <Body1>
                        "Principles are review criteria—not slogans. Before shipping a page, walk the four principles "
                        "and check whether spacing, type, and interaction choices support them. When two implementation "
                        "options are valid, the principle that serves the current task breaks the tie."
                    </Body1>
                </Flex>

                <PrincipleBlock
                    id="principles-familiar"
                    title="Familiar on every surface"
                    class=classes.section
                    body="Interfaces adapt to the device and build on patterns users already know. Invest custom UX only where the product has a signature moment worth learning."
                    practice="Use Orbital shell layouts before inventing new chrome. Prefer responsive padding and reflow over fixed desktop layouts shrunk to mobile. Reach for standard components over one-off styled wrappers."
                    avoid="Novel navigation on every app; hiding core actions behind unique gestures; desktop-only density on phone widths."
                />

                <PrincipleBlock
                    id="principles-focus"
                    title="Built for focus"
                    class=classes.section
                    body="Draw attention to the next action and the information needed for the current task. Remove noise so people stay in flow."
                    practice="Keep the content canvas as the lightest, most prominent surface. One primary action per region; secondary actions use subtle button appearances. Use typography hierarchy—one clear page title."
                    avoid="Dense walls of controls; competing primary buttons; decorative color and motion that do not carry meaning."
                />

                <PrincipleBlock
                    id="principles-inclusive"
                    title="Inclusive by design"
                    class=classes.section
                    body="Design for a range of abilities, preferences, and contexts from the start—not as a retrofit."
                    practice="Never rely on color alone for status; pair with text or icon. Ensure keyboard reachability and visible focus. Respect prefers-reduced-motion; label icon-only controls."
                    avoid="Placeholder-only error states; contrast that fails accessibility targets; motion that cannot be reduced."
                />

                <PrincipleBlock
                    id="principles-cohesive"
                    title="Cohesive product character"
                    class=classes.section
                    body="Apps built with Orbital should feel like parts of one platform: shared tokens, shared motion, shared patterns."
                    practice="Use design tokens for color, spacing, shadow, and radius—no ad-hoc hex in app code. Compose with Material, Stack, and typography presets. Brand accent sparingly on CTAs and selection."
                    avoid="Per-app shadow and radius values; forked copies of shell components; brand color as the default page background."
                />

                <Flex vertical=true gap=SpacingSize::Size120.flex_gap()>
                    <SectionHeading
                        id="principles-glance"
                        title="Principles at a glance"
                        class=classes.section
                    />
                    <div class=classes.table style="grid-template-columns: 1fr 1fr;">
                        <div class=classes.table_header>
                            <div class=classes.table_cell>"Principle"</div>
                            <div class=classes.table_cell>"Primary chapters"</div>
                        </div>
                        <div class=classes.table_row>
                            <div class=classes.table_cell>"Familiar on every surface"</div>
                            <div class=classes.table_cell>"Layout, shell patterns, responsive"</div>
                        </div>
                        <div class=classes.table_row>
                            <div class=classes.table_cell>"Built for focus"</div>
                            <div class=classes.table_cell>"Material, Elevation, Typography"</div>
                        </div>
                        <div class=classes.table_row>
                            <div class=classes.table_cell>"Inclusive by design"</div>
                            <div class=classes.table_cell>"Typography, Color, Motion"</div>
                        </div>
                        <div class=classes.table_row>
                            <div class=classes.table_cell>"Cohesive product character"</div>
                            <div class=classes.table_cell>"Tokens, Material, Typography presets"</div>
                        </div>
                    </div>
                </Flex>
            </Flex>
        </Card>
    }
}

#[component]
pub(super) fn PrincipleBlock(
    /// Identifier.
    id: &'static str,
    /// Title text.
    title: &'static str,
    /// Additional CSS class(es) to apply.
    class: &'static str,
    /// Body text.
    body: &'static str,
    /// Recommended practice description.
    practice: &'static str,
    /// Anti-pattern description to avoid.
    avoid: &'static str,
) -> impl IntoView {
    view! {
        <Flex vertical=true gap=SpacingSize::Size80.flex_gap()>
            <SectionHeading id=id title=title class=class />
            <Body1>{body}</Body1>
            <Caption1>"In practice"</Caption1>
            <Body1>{practice}</Body1>
            <Caption1>"Avoid"</Caption1>
            <Body1>{avoid}</Body1>
        </Flex>
    }
}
