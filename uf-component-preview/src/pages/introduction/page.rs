use leptos::prelude::*;
use uf_product::components::{
    Body1, Card, ContentContainer, SpacingSize, Subtitle1, Subtitle2, Title3,
};
use uf_product::primitives::Flex;

use super::chapters::{
    ColorChapter, ElevationChapter, FurtherReadingFooter, LayoutChapter, MaterialChapter,
    MotionChapter, PrinciplesChapter, TypographyChapter,
};

#[derive(Clone, Copy)]
pub(super) struct TocClassNames {
    pub(super) toc_link: &'static str,
    pub(super) chapter: &'static str,
    pub(super) section: &'static str,
    pub(super) table: &'static str,
    pub(super) table_header: &'static str,
    pub(super) table_row: &'static str,
    pub(super) table_cell: &'static str,
    pub(super) footer: &'static str,
    pub(super) external_link: &'static str,
    pub(super) mono: &'static str,
}

#[component]
pub fn IntroductionPage() -> impl IntoView {
    let (style_sheet, styles) = turf::inline_style_sheet_values! {
        .Page {
            padding-block: var(--spacingVerticalXXL);
        }
        .Chapter {
            scroll-margin-top: var(--spacingVerticalXXL);
        }
        .Section {
            scroll-margin-top: var(--spacingVerticalL);
        }
        .TocLink {
            color: var(--colorBrandForeground1);
            text-decoration: none;
        }
        .TocLink:hover {
            text-decoration: underline;
        }
        .ExternalLink {
            color: var(--colorBrandForeground1);
        }
        .Table {
            display: grid;
            gap: var(--spacingVerticalXS);
            font-size: var(--fontSizeBase300);
            line-height: var(--lineHeightBase300);
        }
        .TableHeader {
            display: contents;
            font-weight: var(--fontWeightSemibold);
            color: var(--colorNeutralForeground2);
        }
        .TableRow {
            display: contents;
        }
        .TableCell {
            padding: var(--spacingVerticalXS) var(--spacingHorizontalS);
            border-bottom: var(--strokeWidthThin) solid var(--colorNeutralStroke2);
        }
        .Footer {
            margin-top: var(--spacingVerticalXXL);
            padding-top: var(--spacingVerticalL);
            border-top: var(--strokeWidthThin) solid var(--colorNeutralStroke2);
        }
        .Mono {
            font-family: var(--fontFamilyMonospace);
            font-size: var(--fontSizeBase200);
        }
    };

    let classes = TocClassNames {
        toc_link: styles.toc_link,
        chapter: styles.chapter,
        section: styles.section,
        table: styles.table,
        table_header: styles.table_header,
        table_row: styles.table_row,
        table_cell: styles.table_cell,
        footer: styles.footer,
        external_link: styles.external_link,
        mono: styles.mono,
    };

    view! {
        <style>{style_sheet}</style>
        <ContentContainer>
            <div class=styles.page data-testid="introduction-page">
                <Flex vertical=true gap=SpacingSize::Size320.flex_gap()>
                    <IntroHero />
                    <InPageToc classes=classes />
                    <PrinciplesChapter classes=classes />
                    <LayoutChapter classes=classes />
                    <ColorChapter classes=classes />
                    <ElevationChapter classes=classes />
                    <MaterialChapter classes=classes />
                    <TypographyChapter classes=classes />
                    <MotionChapter classes=classes />
                    <FurtherReadingFooter classes=classes />
                </Flex>
            </div>
        </ContentContainer>
    }
}

#[component]
fn IntroHero() -> impl IntoView {
    view! {
        <Flex vertical=true gap=SpacingSize::Size120.flex_gap()>
            <Title3>"Introduction"</Title3>
            <Body1>
                "Orbital is a Leptos component library for building focused, accessible product interfaces. "
                "The sections below define spacing, type, color, surfaces, and motion in concrete terms. "
                "Start with four principles that explain why those rules exist."
            </Body1>
            <Body1>
                "Use the table of contents to jump to a chapter, or browse individual components in the sidebar."
            </Body1>
        </Flex>
    }
}

#[component]
fn InPageToc(
    /// Additional CSS class(es) to apply.
    classes: TocClassNames,
) -> impl IntoView {
    view! {
        <Card>
            <Flex vertical=true gap=SpacingSize::Size120.flex_gap() padding=SpacingSize::Size160.inset()>
                <Subtitle2>"On this page"</Subtitle2>
                <Flex vertical=true gap=SpacingSize::Size80.flex_gap()>
                    <a class=classes.toc_link href="#principles">"Principles"</a>
                    <a class=classes.toc_link href="#layout">"Layout"</a>
                    <a class=classes.toc_link href="#color">"Color"</a>
                    <a class=classes.toc_link href="#elevation">"Elevation"</a>
                    <a class=classes.toc_link href="#material">"Material"</a>
                    <a class=classes.toc_link href="#typography">"Typography"</a>
                    <a class=classes.toc_link href="#motion">"Motion"</a>
                </Flex>
            </Flex>
        </Card>
    }
}
#[component]
pub(super) fn ChapterHeading(
    /// Identifier.
    id: &'static str,
    /// Title text.
    title: &'static str,
    /// Additional CSS class(es) to apply.
    class: &'static str,
) -> impl IntoView {
    view! {
        <div id=id class=class>
            <Subtitle1>{title}</Subtitle1>
        </div>
    }
}

#[component]
pub(super) fn SectionHeading(
    /// Identifier.
    id: &'static str,
    /// Title text.
    title: &'static str,
    /// Additional CSS class(es) to apply.
    class: &'static str,
) -> impl IntoView {
    view! {
        <div id=id class=class>
            <Subtitle2>{title}</Subtitle2>
        </div>
    }
}
