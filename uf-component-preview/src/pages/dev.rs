use leptos::prelude::*;
use leptos_router::components::A;
use uf_integrations::UnifiedFieldAppBar;
use uf_product::components::{
    AutoGrid, Body1, Caption1, Card, FlexWrap, Paginator, ScrollArea, SpacingSize, StatCard,
    StatCardVariant, Step, StepStatus, Stepper, Subtitle2, Tag, Title3,
    EMPTYSTATE_SAD_DOG_ILLUSTRATION,
};
use uf_product::primitives::*;

use crate::preview::PreviewCatalogShell;

/// Orbital component preview shell (v0.1.2 catalog layout).
#[component]
pub fn OrbitalDevLayout() -> impl IntoView {
    view! { <PreviewCatalogShell /> }
}

/// Reusable card component for displaying component previews
#[component]
fn ComponentPreviewCard(
    /// Display name of the component
    name: &'static str,
    /// Link to the detailed preview page
    href: &'static str,
    /// Whether to use full height for the preview area
    #[prop(optional)]
    full_height: bool,
    /// The component to preview
    children: Children,
) -> impl IntoView {
    let (style_sheet, class_names) = turf::inline_style_sheet_values! {
        .CardHeader {
            padding: var(--spacingVerticalL) var(--spacingHorizontalXXL);
            background: var(--colorNeutralBackground3);
            border-bottom: var(--strokeWidthThin) solid var(--colorNeutralStroke1);
            display: flex;
            justify-content: space-between;
            align-items: center;
        }

        .Preview {
            padding: 0;
            min-height: 200px;
        }

        .FullHeight {
            height: 400px;
        }
    };

    view! {
        <style>{style_sheet}</style>
        <Card>
            <div class=class_names.card_header>
                <Subtitle2>{name}</Subtitle2>
                <A href=href attr:style="text-decoration: none;">
                    <Button appearance=ButtonAppearance::Subtle size=ButtonSize::Small>
                        "View Component →"
                    </Button>
                </A>
            </div>
            <div class=move || {
                if full_height {
                    format!("{} {}", class_names.preview, class_names.full_height)
                } else {
                    class_names.preview.to_string()
                }
            }>
                {children()}
            </div>
        </Card>
    }
}

/// Component preview page for development
#[component]
pub fn ComponentPreview() -> impl IntoView {
    view! {
        <div data-testid="component-preview-container">
        <Flex
            vertical=true
            gap=SpacingSize::Size160.flex_gap()
        >
            <Flex vertical=true gap=SpacingSize::Size80.flex_gap()>
                <Title3>"Orbital Component Preview"</Title3>
                <Body1>"Browse the Orbital shell components and their previews."</Body1>
            </Flex>

            <ComponentPreviewCard name="App Bar" href="/orbital/app-bar">
                <UnifiedFieldAppBar app_name="Example App".to_string() />
            </ComponentPreviewCard>

            <ComponentPreviewCard name="Text" href="/orbital/text">
                <Flex vertical=true gap=SpacingSize::Size80.flex_gap() padding=SpacingSize::Size160.inset()>
                    <Title3>"Title3 Example"</Title3>
                    <Body1>"Body1 example text"</Body1>
                    <Caption1>"Caption1 example"</Caption1>
                </Flex>
            </ComponentPreviewCard>

            <ComponentPreviewCard name="Scroll Area" href="/orbital/scroll-area">
                <div style="height: 220px; border: var(--strokeWidthThin) solid var(--colorNeutralStroke1); border-radius: var(--borderRadiusXLarge); overflow: hidden;">
                    <ScrollArea style="height: 100%; padding: var(--spacingVerticalL);">
                        <div style="height: 400px;">
                            <Body1>"This content scrolls vertically with a hidden scrollbar."</Body1>
                            <Body1 style="margin-top: 200px;">"Scroll down to see more content..."</Body1>
                            <Body1 style="margin-top: 100px;">"Bottom of scrollable area"</Body1>
                        </div>
                    </ScrollArea>
                </div>
            </ComponentPreviewCard>

            <ComponentPreviewCard name="Search Source Picker" href="/orbital/search-source-picker">
                <div style="padding: var(--spacingVerticalL);">
                    <Body1>"Multi-source user/group search picker"</Body1>
                </div>
            </ComponentPreviewCard>

            <ComponentPreviewCard name="Tag Catalog Picker" href="/orbital/tag-catalog-picker">
                <div style="padding: var(--spacingVerticalL);">
                    <Body1>"Multi-select shared tag catalog"</Body1>
                </div>
            </ComponentPreviewCard>

            <ComponentPreviewCard name="Infinite Scroll" href="/orbital/infinite-scroll">
                <div style="padding: var(--spacingVerticalL);">
                    <Flex vertical=true gap=SpacingSize::Size80.flex_gap()>
                        <Body1>"Paginated infinite-scroll container"</Body1>
                        <Caption1>"Automatic loading, empty states, and end-of-list indicators with customisable slots."</Caption1>
                        <Flex gap=SpacingSize::Size40.flex_gap() wrap=FlexWrap::Wrap>
                            <Tag>"let:items"</Tag>
                            <Tag>"Page<T>"</Tag>
                            <Tag>"Slots"</Tag>
                        </Flex>
                    </Flex>
                </div>
            </ComponentPreviewCard>

            <ComponentPreviewCard name="History Timeline" href="/orbital/history-timeline">
                <div style="padding: var(--spacingVerticalL);">
                    <Flex vertical=true gap=SpacingSize::Size80.flex_gap()>
                        <Body1>"RecordHistory trait + infinite scroll audit rows (newest first)"</Body1>
                        <Caption1>"Embed HistoryTimeline on detail pages; optional kinds filter."</Caption1>
                    </Flex>
                </div>
            </ComponentPreviewCard>

            <ComponentPreviewCard name="Auto Grid" href="/orbital/auto-grid">
                <div style="padding: var(--spacingVerticalL);">
                    <AutoGrid min="160px" gap=SpacingSize::Size120>
                        <Card>
                            <Body1>"Item 1"</Body1>
                        </Card>
                        <Card>
                            <Body1>"Item 2"</Body1>
                        </Card>
                        <Card>
                            <Body1>"Item 3"</Body1>
                        </Card>
                        <Card>
                            <Body1>"Item 4"</Body1>
                        </Card>
                    </AutoGrid>
                </div>
            </ComponentPreviewCard>

            <ComponentPreviewCard name="Stat Card" href="/orbital/stat-card">
                <div style="padding: var(--spacingVerticalL);">
                    <Flex gap=SpacingSize::Size160.flex_gap() wrap=FlexWrap::Wrap>
                        <StatCard
                            label="Total Users"
                            value=Signal::derive(|| "1,234".to_string())
                        />
                        <StatCard
                            label="Active"
                            value=Signal::derive(|| "892".to_string())
                            variant=StatCardVariant::Success
                        />
                        <StatCard
                            label="Errors"
                            value=Signal::derive(|| "5".to_string())
                            variant=StatCardVariant::Danger
                        />
                    </Flex>
                </div>
            </ComponentPreviewCard>

            <ComponentPreviewCard name="Empty State" href="/orbital/empty-state">
                <div style="padding: var(--spacingVerticalL);">
                    <uf_product::components::EmptyState
                        message="No items found"
                        description="Try adjusting your search or filters."
                        illustration_src=EMPTYSTATE_SAD_DOG_ILLUSTRATION
                        illustration_alt="Sad dog with a tear"
                    />
                </div>
            </ComponentPreviewCard>

            <ComponentPreviewCard name="Stepper" href="/orbital/stepper">
                <div style="padding: var(--spacingVerticalL);">
                    <Stepper>
                        <Step slot:steps label="Download" status=StepStatus::Done />
                        <Step slot:steps label="Install" status=StepStatus::Active message="In progress..." />
                        <Step slot:steps label="Configure" status=StepStatus::Pending />
                    </Stepper>
                </div>
            </ComponentPreviewCard>

            <ComponentPreviewCard name="Paginator" href="/orbital/paginator">
                <div style="padding: var(--spacingVerticalL);">
                    {
                        let offset = RwSignal::new(0u32);
                        let total: RwSignal<Option<u64>> = RwSignal::new(Some(250u64));
                        view! {
                            <Paginator offset=offset limit=25 total_count=total />
                        }
                    }
                </div>
            </ComponentPreviewCard>

            <ComponentPreviewCard name="Pattern: Identity Card" href="/orbital/components/patterns/identity-card">
                <div style="padding: var(--spacingVerticalL); max-width: 420px;">
                    <uf_product::components::IdentityCard
                        name="Taylor Reid"
                        title="Core Observability"
                        subtitle="Owns dashboards, SLOs, and incident response runbooks."
                        avatar_size=40
                    />
                </div>
            </ComponentPreviewCard>
        </Flex>
        </div>
    }
}
