use leptos::prelude::*;
use leptos_router::components::A;
use uf_product::components::{Body1, Caption1, Card, SpacingSize};
use uf_product::primitives::Flex;

use super::page::{ChapterHeading, SectionHeading, TocClassNames};

#[component]
pub(super) fn ColorChapter(
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
                <ChapterHeading id="color" title="Color" class=classes.chapter />
                <Body1>
                    "Color expresses style, communicates meaning, and supports hierarchy. "
                    "Orbital organizes color into neutral, shared accent, and brand palettes applied through design tokens."
                </Body1>
                <Body1>
                    "Neutral colors carry surfaces, text, and layout chrome. Shared accent colors highlight reusable components. "
                    "Brand colors identify the product—use them sparingly on large surfaces."
                </Body1>
                <Body1>
                    "Semantic status colors (success, warning, danger) build on real-world associations. "
                    "Always pair them with text or icons—never rely on color alone."
                </Body1>
                <Body1>
                    "Lighter neutrals on primary focus surfaces and darker neutrals on surrounding chrome draw the eye to the task. "
                    "Interaction states progress from rest through hover, pressed, and selected; focus adds a thicker stroke."
                </Body1>
                <Caption1>"Key token families"</Caption1>
                <Body1>
                    <span class=classes.mono>"--colorNeutralBackground*"</span>
                    ", "
                    <span class=classes.mono>"--colorNeutralForeground*"</span>
                    ", "
                    <span class=classes.mono>"--colorBrandBackground"</span>
                    ", "
                    <span class=classes.mono>"--colorStatusSuccessForeground1"</span>
                    ", and related status tokens."
                </Body1>
            </Flex>
        </Card>
    }
}

#[component]
pub(super) fn ElevationChapter(
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
                <ChapterHeading id="elevation" title="Elevation" class=classes.chapter />

                <Flex vertical=true gap=SpacingSize::Size120.flex_gap()>
                    <SectionHeading
                        id="elevation-depth"
                        title="Depth and shadow"
                        class=classes.section
                    />
                    <Body1>
                        "Elevation is a hierarchy signal. A resting panel, a raised card, a dropdown, and a modal dialog each occupy a different depth. "
                        "Orbital encodes depth with named shadow tokens so surfaces feel coherent instead of each team tuning shadows by eye."
                    </Body1>
                </Flex>

                <Flex vertical=true gap=SpacingSize::Size120.flex_gap()>
                    <SectionHeading
                        id="elevation-ramp"
                        title="Elevation ramp"
                        class=classes.section
                    />
                    <div class=classes.table style="grid-template-columns: 1fr 1.5fr;">
                        <div class=classes.table_header>
                            <div class=classes.table_cell>"Token"</div>
                            <div class=classes.table_cell>"Typical use"</div>
                        </div>
                        <div class=classes.table_row>
                            <div class=classes.table_cell>
                                <span class=classes.mono>"--shadow4"</span>
                            </div>
                            <div class=classes.table_cell>"Cards, list items, content canvas, TopBar"</div>
                        </div>
                        <div class=classes.table_row>
                            <div class=classes.table_cell>
                                <span class=classes.mono>"--shadow8"</span>
                            </div>
                            <div class=classes.table_cell>"Emphasized cards, raised command bars"</div>
                        </div>
                        <div class=classes.table_row>
                            <div class=classes.table_cell>
                                <span class=classes.mono>"--shadow16"</span>
                            </div>
                            <div class=classes.table_cell>"Callouts, transient sidebar panels"</div>
                        </div>
                        <div class=classes.table_row>
                            <div class=classes.table_cell>
                                <span class=classes.mono>"--shadow64"</span>
                            </div>
                            <div class=classes.table_cell>"Dialogs and modal panels"</div>
                        </div>
                    </div>
                    <Body1>
                        "Match elevation to how long the surface stays and how much it blocks what is beneath. "
                        "Persistent page content stays at resting elevation. Blocking dialogs sit at the top of the ramp."
                    </Body1>
                </Flex>

                <Flex vertical=true gap=SpacingSize::Size120.flex_gap()>
                    <SectionHeading
                        id="elevation-shell"
                        title="Shell conventions"
                        class=classes.section
                    />
                    <div class=classes.table style="grid-template-columns: 1fr 1fr;">
                        <div class=classes.table_header>
                            <div class=classes.table_cell>"Region"</div>
                            <div class=classes.table_cell>"Elevation"</div>
                        </div>
                        <div class=classes.table_row>
                            <div class=classes.table_cell>"TopBar"</div>
                            <div class=classes.table_cell>"Resting (shadow4)"</div>
                        </div>
                        <div class=classes.table_row>
                            <div class=classes.table_cell>"Content canvas"</div>
                            <div class=classes.table_cell>"Resting (shadow4)"</div>
                        </div>
                        <div class=classes.table_row>
                            <div class=classes.table_cell>"Right sidebar (open)"</div>
                            <div class=classes.table_cell>"Floating (shadow16)"</div>
                        </div>
                        <div class=classes.table_row>
                            <div class=classes.table_cell>"Dialogs"</div>
                            <div class=classes.table_cell>"Modal (shadow64)"</div>
                        </div>
                    </div>
                </Flex>
            </Flex>
        </Card>
    }
}

#[component]
pub(super) fn MaterialChapter(
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
                <ChapterHeading id="material" title="Material" class=classes.chapter />

                <Body1>
                    "Material describes what a surface feels like—opaque workspace, frosted glass, tinted backdrop, or dimmed scrim—not how far it floats. "
                    "Depth is elevation (see above). Pick variant first, then elevation where shadow applies."
                </Body1>

                <Flex vertical=true gap=SpacingSize::Size120.flex_gap()>
                    <SectionHeading id="material-solid" title="Solid" class=classes.section />
                    <Body1>
                        "Solid material is the default for anything that stays on screen while the user works—page body, cards, nav rails. "
                        "Differentiate regions with background step tokens and elevation, not one-off hex fills."
                    </Body1>
                    <div class=classes.table style="grid-template-columns: 1fr 1.5fr;">
                        <div class=classes.table_header>
                            <div class=classes.table_cell>"Region"</div>
                            <div class=classes.table_cell>"Background token"</div>
                        </div>
                        <div class=classes.table_row>
                            <div class=classes.table_cell>"Shell ground"</div>
                            <div class=classes.table_cell>
                                <span class=classes.mono>"--colorNeutralBackground3"</span>
                            </div>
                        </div>
                        <div class=classes.table_row>
                            <div class=classes.table_cell>"Content canvas"</div>
                            <div class=classes.table_cell>
                                <span class=classes.mono>"--colorNeutralBackground1"</span>
                            </div>
                        </div>
                        <div class=classes.table_row>
                            <div class=classes.table_cell>"Top bar"</div>
                            <div class=classes.table_cell>
                                <span class=classes.mono>"--colorNeutralBackground1"</span>
                            </div>
                        </div>
                    </div>
                </Flex>

                <Flex vertical=true gap=SpacingSize::Size120.flex_gap()>
                    <SectionHeading id="material-acrylic" title="Acrylic" class=classes.section />
                    <Body1>
                        "Acrylic surfaces are semi-transparent with a frosted backdrop. "
                        "Use them for light-dismiss, transient overlays—menus and popovers—not for primary reading surfaces."
                    </Body1>
                </Flex>

                <Flex vertical=true gap=SpacingSize::Size120.flex_gap()>
                    <SectionHeading id="material-smoke" title="Smoke" class=classes.section />
                    <Body1>
                        "Smoke is a dimmed scrim that blocks interaction with the page beneath. "
                        "Pair it with dialog content at modal elevation. Always provide a clear dismiss path and move keyboard focus into the elevated content."
                    </Body1>
                </Flex>

                <Flex vertical=true gap=SpacingSize::Size120.flex_gap()>
                    <SectionHeading
                        id="material-component"
                        title="The Material component"
                        class=classes.section
                    />
                    <Body1>
                        "The Material component declares surface finish and depth in one place. "
                        "Set variant to opaque, frosted, tinted, or dimmed. Set elevation when the surface should lift off its parent."
                    </Body1>
                    <Body1>
                        <A href="/orbital/material">"View Material preview →"</A>
                    </Body1>
                </Flex>
            </Flex>
        </Card>
    }
}
