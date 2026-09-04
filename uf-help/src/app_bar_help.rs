//! App-bar Help control: AdaptiveMenu with report + replay actions.

use leptos::prelude::*;
use leptos_router::hooks::use_location;
use uf_product::primitives::{
    AdaptiveMenu, AdaptiveMenuTrigger, Button, ButtonAppearance, Divider, Stack, Tooltip,
};
use uf_product::AppBarUtilityContribution;

use crate::report::{HelpReportDialog, HelpReportKind};
use crate::tour::request_replay_current_route;

/// Sort order for the default Help control in the app-bar utilities pack.
pub const APP_BAR_UTILITY_ORDER: u8 = 10;

/// Help control for the product app bar — Bug / Feature / Security / Replay.
#[component]
pub fn AppBarHelpButton() -> impl IntoView {
    let location = use_location();
    let bug_open = RwSignal::new(false);
    let feature_open = RwSignal::new(false);
    let security_open = RwSignal::new(false);

    let open_bug = Callback::new(move |_| bug_open.set(true));
    let open_feature = Callback::new(move |_| feature_open.set(true));
    let open_security = Callback::new(move |_| security_open.set(true));
    let replay = Callback::new(move |_| {
        let route = location.pathname.get_untracked();
        request_replay_current_route(route);
    });

    view! {
        <div data-testid="app-bar-help">
            <AdaptiveMenu drawer_aria_label="Help">
                <AdaptiveMenuTrigger slot>
                    <Tooltip content="Help">
                        <Button
                            appearance=ButtonAppearance::Subtle
                            icon=icondata::AiQuestionCircleOutlined
                            attr:aria-label="Help"
                        />
                    </Tooltip>
                </AdaptiveMenuTrigger>
                <div data-testid="help-menu-panel">
                    <Stack>
                        <Button
                            appearance=ButtonAppearance::Subtle
                            block=true
                            on_click=open_bug
                            attr:data-testid="help-menu-report-bug"
                        >
                            "Report a bug"
                        </Button>
                        <Button
                            appearance=ButtonAppearance::Subtle
                            block=true
                            on_click=open_feature
                            attr:data-testid="help-menu-request-feature"
                        >
                            "Request a feature"
                        </Button>
                        <Button
                            appearance=ButtonAppearance::Subtle
                            block=true
                            on_click=open_security
                            attr:data-testid="help-menu-report-security"
                        >
                            "Report a security issue"
                        </Button>
                        <Divider />
                        <Button
                            appearance=ButtonAppearance::Subtle
                            block=true
                            on_click=replay
                            attr:data-testid="help-menu-replay-tour"
                        >
                            "Replay spotlight tour"
                        </Button>
                    </Stack>
                </div>
            </AdaptiveMenu>

            <HelpReportDialog open=bug_open kind=HelpReportKind::Bug />
            <HelpReportDialog open=feature_open kind=HelpReportKind::Feature />
            <HelpReportDialog open=security_open kind=HelpReportKind::Security />
        </div>
    }
}

fn render_help_utility() -> AnyView {
    view! { <AppBarHelpButton /> }.into_any()
}

inventory::submit! {
    AppBarUtilityContribution::new(APP_BAR_UTILITY_ORDER, "help", render_help_utility)
}
