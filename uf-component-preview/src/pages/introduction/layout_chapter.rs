use leptos::prelude::*;
use leptos_router::components::A;
use uf_product::components::{Body1, Caption1, Card, SpacingSize};
use uf_product::primitives::Flex;

use super::page::{ChapterHeading, SectionHeading, TocClassNames};

#[component]
pub(super) fn LayoutChapter(
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
                <ChapterHeading id="layout" title="Layout" class=classes.chapter />

                <Flex vertical=true gap=SpacingSize::Size120.flex_gap()>
                    <SectionHeading
                        id="layout-spacing"
                        title="Spacing and proximity"
                        class=classes.section
                    />
                    <Body1>
                        "Space is a grouping tool. When two controls share a small gap, people read them as one decision. "
                        "When sections are separated by a full ramp step or more, the page reads in clear chunks without divider lines."
                    </Body1>
                    <Body1>
                        "Orbital constrains gaps to "
                        <span class=classes.mono>"SpacingSize"</span>
                        " and CSS spacing tokens—avoid one-off margin values that break rhythm across pages."
                    </Body1>
                    <Caption1>"Common spacing ramp values"</Caption1>
                    <div class=classes.table style="grid-template-columns: 1fr 1fr;">
                        <div class=classes.table_header>
                            <div class=classes.table_cell>"Token"</div>
                            <div class=classes.table_cell>"Pixels"</div>
                        </div>
                        <div class=classes.table_row>
                            <div class=classes.table_cell>"Size40"</div>
                            <div class=classes.table_cell>"4px"</div>
                        </div>
                        <div class=classes.table_row>
                            <div class=classes.table_cell>"Size80"</div>
                            <div class=classes.table_cell>"8px"</div>
                        </div>
                        <div class=classes.table_row>
                            <div class=classes.table_cell>"Size120"</div>
                            <div class=classes.table_cell>"12px"</div>
                        </div>
                        <div class=classes.table_row>
                            <div class=classes.table_cell>"Size160 (default)"</div>
                            <div class=classes.table_cell>"16px"</div>
                        </div>
                        <div class=classes.table_row>
                            <div class=classes.table_cell>"Size240"</div>
                            <div class=classes.table_cell>"24px"</div>
                        </div>
                        <div class=classes.table_row>
                            <div class=classes.table_cell>"Size320"</div>
                            <div class=classes.table_cell>"32px"</div>
                        </div>
                    </div>
                    <Body1>
                        "Shell content padding is responsive: 32px desktop, 24px tablet, 16px mobile. "
                        "Regions in the shell body use a 12px gap so chrome does not butt together."
                    </Body1>
                </Flex>

                <Flex vertical=true gap=SpacingSize::Size120.flex_gap()>
                    <SectionHeading
                        id="layout-components"
                        title="Choosing a layout component"
                        class=classes.section
                    />
                    <Body1>
                        "Start with the simplest primitive that fits. "
                        <span class=classes.mono>"Stack"</span>
                        " is the default for vertical sections and evenly spaced rows. "
                        "Reach for "
                        <span class=classes.mono>"Flex"</span>
                        " when you need alignment along both axes or inline placement."
                    </Body1>
                    <div class=classes.table style="grid-template-columns: 1.2fr 2fr;">
                        <div class=classes.table_header>
                            <div class=classes.table_cell>"Component"</div>
                            <div class=classes.table_cell>"Use when"</div>
                        </div>
                        <div class=classes.table_row>
                            <div class=classes.table_cell>
                                <A href="/orbital/box">"Box"</A>
                            </div>
                            <div class=classes.table_cell>"One wrapper needs token-based padding or sizing—not sibling distribution"</div>
                        </div>
                        <div class=classes.table_row>
                            <div class=classes.table_cell>
                                <A href="/orbital/stack">"Stack"</A>
                            </div>
                            <div class=classes.table_cell>"Multiple children, one axis, even gaps (forms, button rows)"</div>
                        </div>
                        <div class=classes.table_row>
                            <div class=classes.table_cell>
                                <A href="/orbital/flex">"Flex"</A>
                            </div>
                            <div class=classes.table_cell>"Alignment, wrapping, or inline control groups beside text"</div>
                        </div>
                        <div class=classes.table_row>
                            <div class=classes.table_cell>
                                <A href="/orbital/auto-grid">"AutoGrid"</A>
                            </div>
                            <div class=classes.table_cell>"Fluid card walls that reflow by viewport without manual breakpoints"</div>
                        </div>
                        <div class=classes.table_row>
                            <div class=classes.table_cell>"Container"</div>
                            <div class=classes.table_cell>"Page-level max-width and horizontal centering inside the shell"</div>
                        </div>
                    </div>
                </Flex>

                <Flex vertical=true gap=SpacingSize::Size120.flex_gap()>
                    <SectionHeading id="layout-grid" title="Grid" class=classes.section />
                    <Body1>
                        "A column grid splits the content area into even tracks—use "
                        <span class=classes.mono>"Grid"</span>
                        " and "
                        <span class=classes.mono>"GridItem"</span>
                        " when column span matters. When tile count and width vary, "
                        <span class=classes.mono>"AutoGrid"</span>
                        " reflows columns using minmax so you do not maintain breakpoint-specific column counts by hand."
                    </Body1>
                </Flex>

                <Flex vertical=true gap=SpacingSize::Size120.flex_gap()>
                    <SectionHeading id="layout-alignment" title="Alignment" class=classes.section />
                    <Body1>
                        "Misaligned baselines make forms feel broken faster than wrong colors. "
                        "In a row of controls, align on the cross axis with Flex or Stack align props. "
                        "Pair icons and labels by centering the glyph and start-aligning the label text."
                    </Body1>
                </Flex>

                <Flex vertical=true gap=SpacingSize::Size120.flex_gap()>
                    <SectionHeading
                        id="layout-responsive"
                        title="Responsive design"
                        class=classes.section
                    />
                    <Body1>
                        "Responsive layout is a set of techniques: reposition (stack to row), resize (container max-width), "
                        "reflow (AutoGrid columns), show/hide (nav collapse), and re-architect (master/detail split on desktop, single pane on mobile)."
                    </Body1>
                    <Caption1>"Breakpoint reference"</Caption1>
                    <div class=classes.table style="grid-template-columns: 1fr 1fr 1fr;">
                        <div class=classes.table_header>
                            <div class=classes.table_cell>"Size class"</div>
                            <div class=classes.table_cell>"Range"</div>
                            <div class=classes.table_cell>"Breakpoint"</div>
                        </div>
                        <div class=classes.table_row>
                            <div class=classes.table_cell>"Small"</div>
                            <div class=classes.table_cell>"320–479px"</div>
                            <div class=classes.table_cell>"< 480px"</div>
                        </div>
                        <div class=classes.table_row>
                            <div class=classes.table_cell>"Large"</div>
                            <div class=classes.table_cell>"640–1023px"</div>
                            <div class=classes.table_cell>"< 1024px"</div>
                        </div>
                        <div class=classes.table_row>
                            <div class=classes.table_cell>"X-Large"</div>
                            <div class=classes.table_cell>"1024px+"</div>
                            <div class=classes.table_cell>"≥ 1024px"</div>
                        </div>
                    </div>
                </Flex>
            </Flex>
        </Card>
    }
}
