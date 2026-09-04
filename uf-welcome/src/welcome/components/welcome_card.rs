use leptos::children::{Children, ViewFnOnce};
use leptos::prelude::*;
use turf::inline_style_sheet_values;
use uf_product::components::{
    Caption1, Card, CardContent, CardFooter, CardHeader, CardHeaderDescription, CardSectionBorder,
    Title3,
};

/// Reusable card component for welcome page sections.
///
/// Provides a consistent structure with title, subtitle, body content, and optional footer.
#[component]
pub fn WelcomeCard(
    /// Card title (displayed in header)
    title: &'static str,
    /// Card subtitle/description (displayed in header)
    subtitle: &'static str,
    /// Main card body content
    children: Children,
    /// Optional footer content (typically action buttons)
    #[prop(optional, into)]
    footer: Option<ViewFnOnce>,
) -> impl IntoView {
    let (style_sheet, class_names) = inline_style_sheet_values! {
        // Thaw Card defaults to `width: 720px; margin: auto;`
        // In a grid that can cause overflow past the padded container.
        // Force cards to fill their grid cell.
        .Card {
            width: 100%;
            max-width: 100%;
            margin: 0;
            box-sizing: border-box;
        }

        .CardBody {
            display: flex;
            flex-direction: column;
            gap: 8px;
            flex-grow: 1;
        }
    };

    view! {
        <style>{style_sheet}</style>
        <Card class=class_names.card>
            <CardHeader>
                <Title3>{title}</Title3>
                <CardHeaderDescription slot>
                    <Caption1>{subtitle}</Caption1>
                </CardHeaderDescription>
            </CardHeader>
            <CardContent class=class_names.card_body>
                {children()}
            </CardContent>
            {footer.map(|footer_fn| {
                view! {
                    <>
                        <CardSectionBorder />
                        <CardFooter>
                            {footer_fn.run()}
                        </CardFooter>
                    </>
                }
            })}
        </Card>
    }
}
