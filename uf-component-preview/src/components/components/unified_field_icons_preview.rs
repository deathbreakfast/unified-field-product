use leptos::prelude::*;
use orbital_shell::icons::Orbital;
use uf_product::components::{
    AutoGrid, Body1, Caption2, ComponentPreviewCard, OrbitalComponentView, OrbitalPreviewCardBody,
    SpacingSize,
};
use uf_product::primitives::{Flex, FlexAlign, Icon};

/// Unified Field icons preview page (shell Orbital mark + product nav icon stand-ins).
#[component]
pub fn UnifiedFieldIconsPreview() -> impl IntoView {
    let products = [
        ("Unified Field", icondata::AiAppstoreOutlined),
        ("Valence", icondata::AiDatabaseOutlined),
        ("Chronon", icondata::AiClockCircleOutlined),
        ("Boson", icondata::AiThunderboltOutlined),
        ("Photon", icondata::AiWifiOutlined),
        ("Spectra", icondata::AiBarChartOutlined),
        ("Gluon", icondata::AiDeploymentUnitOutlined),
    ];

    view! {
        <OrbitalComponentView
            component_name="Unified Field Icons"
            component_description="Shell Orbital mark and icondata stand-ins for product navigation. Custom brand SVG marks are product-owned and not part of upstream Orbital v0.2.0."
            default=view! {
                <OrbitalPreviewCardBody code="<Orbital />">
                    <Flex align=FlexAlign::Center gap=SpacingSize::Size160.flex_gap()>
                        <Orbital />
                        <Body1>"Orbital shell mark"</Body1>
                    </Flex>
                </OrbitalPreviewCardBody>
            }
        >
            <ComponentPreviewCard
                title="Product navigation icons"
                code=r#"<Icon icon=icondata::AiDatabaseOutlined />"#>
                <AutoGrid min="120px" gap=SpacingSize::Size240>
                    {products.into_iter().map(|(label, icon)| view! {
                        <Flex vertical=true align=FlexAlign::Center gap=SpacingSize::Size80.flex_gap()>
                            <Icon icon=icon width="32px" height="32px" />
                            <Caption2>{label}</Caption2>
                        </Flex>
                    }).collect_view()}
                </AutoGrid>
            </ComponentPreviewCard>
        </OrbitalComponentView>
    }
}
