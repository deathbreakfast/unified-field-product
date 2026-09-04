use leptos::prelude::*;
use leptos_router::components::A;
use uf_product::components::{Body1, Caption1, Card, SpacingSize, Subtitle2};
use uf_product::primitives::{Divider, Flex};

use super::page::{ChapterHeading, SectionHeading, TocClassNames};

#[component]
pub(super) fn TypographyChapter(
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
                <ChapterHeading id="typography" title="Typography" class=classes.chapter />

                <Flex vertical=true gap=SpacingSize::Size120.flex_gap()>
                    <SectionHeading
                        id="typography-ramp"
                        title="Hierarchy and type ramp"
                        class=classes.section
                    />
                    <Body1>
                        "The type ramp is a ladder of paired font size, line height, and weight presets. "
                        "Default reading text is Body1; scale up for titles, down for metadata."
                    </Body1>
                    <div class=classes.table style="grid-template-columns: 1fr 1fr 2fr;">
                        <div class=classes.table_header>
                            <div class=classes.table_cell>"Preset"</div>
                            <div class=classes.table_cell>"Size"</div>
                            <div class=classes.table_cell>"Typical use"</div>
                        </div>
                        <div class=classes.table_row>
                            <div class=classes.table_cell>"Caption1"</div>
                            <div class=classes.table_cell>"12px"</div>
                            <div class=classes.table_cell>"Metadata, timestamps"</div>
                        </div>
                        <div class=classes.table_row>
                            <div class=classes.table_cell>"Body1"</div>
                            <div class=classes.table_cell>"14px"</div>
                            <div class=classes.table_cell>"Default body text"</div>
                        </div>
                        <div class=classes.table_row>
                            <div class=classes.table_cell>"Subtitle2"</div>
                            <div class=classes.table_cell>"16px semibold"</div>
                            <div class=classes.table_cell>"Card subtitles, section headers"</div>
                        </div>
                        <div class=classes.table_row>
                            <div class=classes.table_cell>"Title3"</div>
                            <div class=classes.table_cell>"24px semibold"</div>
                            <div class=classes.table_cell>"Page titles in the app shell"</div>
                        </div>
                        <div class=classes.table_row>
                            <div class=classes.table_cell>"Display"</div>
                            <div class=classes.table_cell>"68px semibold"</div>
                            <div class=classes.table_cell>"Hero and marketing statements"</div>
                        </div>
                    </div>
                    <Body1>
                        <A href="/orbital/text">"View full type specimen →"</A>
                    </Body1>
                </Flex>

                <Flex vertical=true gap=SpacingSize::Size120.flex_gap()>
                    <SectionHeading
                        id="typography-presets"
                        title="Choosing a preset"
                        class=classes.section
                    />
                    <Body1>
                        "If you reach for a raw element with inline font styles, check the ramp first—there is almost always a preset that fits. "
                        "Use FormLabel for field labels, FormHint for helper text below fields, and SectionTitle for compact group headings in dense settings UI."
                    </Body1>
                </Flex>

                <Flex vertical=true gap=SpacingSize::Size120.flex_gap()>
                    <SectionHeading
                        id="typography-styling"
                        title="Styling text"
                        class=classes.section
                    />
                    <Body1>
                        "Use sentence case for UI strings. Default start alignment for paragraphs and forms; center only for short, intentional focus. "
                        "Typography carries meaning through size and weight first, color second—muted captions use foreground tokens, not random gray hex. "
                        "Body text needs sufficient contrast against its background."
                    </Body1>
                </Flex>
            </Flex>
        </Card>
    }
}

#[component]
pub(super) fn MotionChapter(
    /// Additional CSS class(es) to apply.
    classes: TocClassNames,
) -> impl IntoView {
    view! {
        <Card>
            <Flex
                vertical=true
                gap=SpacingSize::Size160.flex_gap()
                padding=SpacingSize::Size320.inset()
            >
                <ChapterHeading id="motion" title="Motion" class=classes.chapter />
                <Body1>
                    "Motion guides attention and confirms actions—it should never distract from the task. "
                    "Use theme duration tokens ("
                    <span class=classes.mono>"--durationNormal"</span>
                    ", "
                    <span class=classes.mono>"--durationGentle"</span>
                    ") and easing curves ("
                    <span class=classes.mono>"--curveEasyEase"</span>
                    ") instead of arbitrary transition values."
                </Body1>
                <Body1>
                    "Respect "
                    <span class=classes.mono>"prefers-reduced-motion"</span>
                    ". Orbital motion helpers such as "
                    <A href="/orbital/hide-on-scroll">"HideOnScroll"</A>
                    ", "
                    <A href="/orbital/parallax-container">"ParallaxContainer"</A>
                    ", and "
                    <span class=classes.mono>"use_reduced_motion"</span>
                    " honor reduced-motion preferences."
                </Body1>
                <Body1>
                    "Shell transitions (sidebar open/close, dialog enter/exit) should feel quick and predictable. "
                    "Decorative motion belongs in marketing surfaces, not routine data entry."
                </Body1>
            </Flex>
        </Card>
    }
}

#[component]
pub(super) fn FurtherReadingFooter(
    /// Additional CSS class(es) to apply.
    classes: TocClassNames,
) -> impl IntoView {
    view! {
        <div class=classes.footer>
            <Flex vertical=true gap=SpacingSize::Size120.flex_gap()>
                <Subtitle2>"Further reading"</Subtitle2>
                <Caption1>
                    "Orbital design language is inspired by contemporary design-system thinking. External references for comparison:"
                </Caption1>
                <Flex vertical=true gap=SpacingSize::Size80.flex_gap()>
                    <a
                        class=classes.external_link
                        href="https://fluent2.microsoft.design/design-principles"
                        target="_blank"
                        rel="noopener noreferrer"
                    >
                        "Design principles (external)"
                    </a>
                    <a
                        class=classes.external_link
                        href="https://fluent2.microsoft.design/layout"
                        target="_blank"
                        rel="noopener noreferrer"
                    >
                        "Layout guidance (external)"
                    </a>
                    <a
                        class=classes.external_link
                        href="https://fluent2.microsoft.design/elevation"
                        target="_blank"
                        rel="noopener noreferrer"
                    >
                        "Elevation guidance (external)"
                    </a>
                    <a
                        class=classes.external_link
                        href="https://fluent2.microsoft.design/material"
                        target="_blank"
                        rel="noopener noreferrer"
                    >
                        "Material guidance (external)"
                    </a>
                    <a
                        class=classes.external_link
                        href="https://fluent2.microsoft.design/typography"
                        target="_blank"
                        rel="noopener noreferrer"
                    >
                        "Typography guidance (external)"
                    </a>
                </Flex>
                <Divider />
                <Body1>
                    "Browse the shell component gallery at "
                    <A href="/orbital/shell">"/orbital/shell"</A>
                    " or pick a component category in the sidebar."
                </Body1>
            </Flex>
        </div>
    }
}
