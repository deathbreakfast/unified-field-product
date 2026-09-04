use leptos::prelude::*;
use uf_product::primitives::{Button, ButtonAppearance, Text, TextTag};

/// Layout example: a plugin settings panel (preview-only placeholder).
#[component]
pub fn PluginSettingsExample() -> impl IntoView {
    let (style_sheet, class_names) = turf::inline_style_sheet_values! {
        .Wrapper {
            display: flex;
            flex-direction: column;
            gap: 8px;
        }
    };

    view! {
        <style>{style_sheet}</style>
        <div class=class_names.wrapper>
            <Text tag=TextTag::H2>"Plugin Settings"</Text>
            <Button appearance=ButtonAppearance::Secondary>"Activity Feed · Enabled"</Button>
            <Button appearance=ButtonAppearance::Secondary>"Team Chat · Enabled"</Button>
            <Button appearance=ButtonAppearance::Subtle>"Calendar Sync · Disabled"</Button>
            <Button appearance=ButtonAppearance::Subtle>"Beta Plugins · Disabled"</Button>
        </div>
    }
}
