//! Shared multi-step Help report dialog shell.

use leptos::prelude::*;
use leptos_router::hooks::use_location;
use uf_product::components::{Step, StepStatus, Stepper};
use uf_product::primitives::{
    Button, ButtonAppearance, Dialog, DialogActions, DialogBody, DialogContent, DialogSurface,
    DialogTitle, Flex, FlexGap, Link, MessageBar, MessageBarIntent, OpenBind,
};

use super::bug_form::BugFormFields;
use super::feature_form::FeatureFormFields;
use super::intro::github_deep_link;
use super::security_form::SecurityFormFields;
use crate::server::{
    help_repository_for_route, submit_help_bug_report, submit_help_feature_request,
    submit_help_security_report,
};

/// Which Help report flow to open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpReportKind {
    /// Public bug issue.
    Bug,
    /// Public feature / enhancement issue.
    Feature,
    /// Private vulnerability report.
    Security,
}

impl HelpReportKind {
    fn title(self) -> &'static str {
        match self {
            Self::Bug => "Report a bug",
            Self::Feature => "Request a feature",
            Self::Security => "Report a security issue",
        }
    }
}

/// Multi-step Orbital dialog: GitHub intro → no-account form → submit.
#[component]
pub fn HelpReportDialog(
    /// Host-owned open signal.
    #[prop(into)]
    open: RwSignal<bool>,
    /// Report kind.
    kind: HelpReportKind,
) -> impl IntoView {
    let location = use_location();
    let step = RwSignal::new(0u8);
    let error = RwSignal::new(Option::<String>::None);
    let success = RwSignal::new(Option::<String>::None);
    let submitting = RwSignal::new(false);
    let open_bind: OpenBind = open.into();

    // Form fields
    let title = RwSignal::new(String::new());
    let description = RwSignal::new(String::new());
    let steps_to_repro = RwSignal::new(String::new());
    let expected = RwSignal::new(String::new());
    let actual = RwSignal::new(String::new());
    let app_version = RwSignal::new(String::new());
    let browser_os = RwSignal::new(String::new());
    let contact = RwSignal::new(String::new());
    let problem = RwSignal::new(String::new());
    let proposed = RwSignal::new(String::new());
    let alternatives = RwSignal::new(String::new());
    let summary = RwSignal::new(String::new());
    let repro = RwSignal::new(String::new());
    let affected = RwSignal::new(String::new());
    let severity = RwSignal::new(String::new());

    let repo_resource = Resource::new(
        move || (open.get(), location.pathname.get()),
        |(is_open, pathname)| async move {
            if !is_open {
                return None;
            }
            help_repository_for_route(pathname).await.ok().flatten()
        },
    );

    Effect::new(move |_| {
        if !open.get() {
            step.set(0);
            error.set(None);
            success.set(None);
            submitting.set(false);
        }
    });

    let close = Callback::new(move |_| open.set(false));

    view! {
        <Dialog open=open_bind>
            <DialogSurface>
                <div data-testid=format!("help-report-dialog-{}", match kind {
                    HelpReportKind::Bug => "bug",
                    HelpReportKind::Feature => "feature",
                    HelpReportKind::Security => "security",
                })>
                    <DialogBody>
                        <DialogTitle>{kind.title()}</DialogTitle>
                        <DialogContent>
                            <Flex vertical=true gap=FlexGap::Medium>
                                {move || {
                                    let s = step.get();
                                    let github = if s == 0 {
                                        StepStatus::Active
                                    } else {
                                        StepStatus::Done
                                    };
                                    let details = if s == 0 {
                                        StepStatus::Pending
                                    } else {
                                        StepStatus::Active
                                    };
                                    view! {
                                        <Stepper vertical=false>
                                            <Step slot:steps label="GitHub" status=github />
                                            <Step slot:steps label="Details" status=details />
                                        </Stepper>
                                    }
                                }}

                                {move || {
                                    error.get().map(|msg| view! {
                                        <MessageBar intent=MessageBarIntent::Error>
                                            {msg}
                                        </MessageBar>
                                    })
                                }}
                                {move || {
                                    success.get().map(|msg| view! {
                                        <MessageBar intent=MessageBarIntent::Success>
                                            {msg}
                                        </MessageBar>
                                    })
                                }}

                                {move || {
                                    if step.get() == 0 {
                                        match repo_resource.get() {
                                            None => view! {
                                                <p data-testid="help-report-repo-loading">
                                                    "Looking up where to file this…"
                                                </p>
                                            }
                                            .into_any(),
                                            Some(None) => view! {
                                                <Flex vertical=true gap=FlexGap::Small>
                                                    <p>
                                                        {match kind {
                                                            HelpReportKind::Bug | HelpReportKind::Feature => {
                                                                "File this on GitHub so maintainers can track it. If you have a GitHub account, open the form there."
                                                            }
                                                            HelpReportKind::Security => {
                                                                "Do not use public issues for security reports. Prefer the private advisory channel."
                                                            }
                                                        }}
                                                    </p>
                                                    <div data-testid="help-report-repo-missing">
                                                        <MessageBar intent=MessageBarIntent::Error>
                                                            "Reporting is not configured for this app (no repository on the app registration)."
                                                        </MessageBar>
                                                    </div>
                                                </Flex>
                                            }
                                            .into_any(),
                                            Some(Some(repo_url)) => {
                                                let link = github_deep_link(kind, &repo_url);
                                                view! {
                                                    <Flex vertical=true gap=FlexGap::Small>
                                                        <p>
                                                            {match kind {
                                                                HelpReportKind::Bug | HelpReportKind::Feature => {
                                                                    "File this on GitHub so maintainers can track it. If you have a GitHub account, open the form there."
                                                                }
                                                                HelpReportKind::Security => {
                                                                    "Do not use public issues for security reports. Prefer the private advisory channel."
                                                                }
                                                            }}
                                                        </p>
                                                        {match link {
                                                            Some(href) => view! {
                                                                <span data-testid="help-report-open-github">
                                                                    <Link href=href inline=true>
                                                                        "Open GitHub"
                                                                    </Link>
                                                                </span>
                                                                <p>"No GitHub account? Continue here and we will file it for you."</p>
                                                            }.into_any(),
                                                            None => view! {
                                                                <div data-testid="help-report-repo-invalid">
                                                                    <MessageBar intent=MessageBarIntent::Error>
                                                                        "Could not build a GitHub link from the app repository URL."
                                                                    </MessageBar>
                                                                </div>
                                                            }.into_any(),
                                                        }}
                                                    </Flex>
                                                }
                                                .into_any()
                                            }
                                        }
                                    } else {
                                        match kind {
                                            HelpReportKind::Bug => view! {
                                                <BugFormFields
                                                    title=title
                                                    description=description
                                                    steps_to_repro=steps_to_repro
                                                    expected=expected
                                                    actual=actual
                                                    app_version=app_version
                                                    browser_os=browser_os
                                                    contact=contact
                                                />
                                            }.into_any(),
                                            HelpReportKind::Feature => view! {
                                                <FeatureFormFields
                                                    title=title
                                                    problem=problem
                                                    proposed=proposed
                                                    alternatives=alternatives
                                                    contact=contact
                                                />
                                            }.into_any(),
                                            HelpReportKind::Security => view! {
                                                <SecurityFormFields
                                                    summary=summary
                                                    description=description
                                                    repro=repro
                                                    affected=affected
                                                    severity=severity
                                                    contact=contact
                                                />
                                            }.into_any(),
                                        }
                                    }
                                }}
                            </Flex>
                        </DialogContent>
                        <DialogActions>
                            {move || {
                                if step.get() == 0 {
                                    view! {
                                        <Button
                                            appearance=ButtonAppearance::Secondary
                                            disabled=Signal::derive(move || {
                                                !matches!(repo_resource.get(), Some(Some(_)))
                                            })
                                            on_click=Callback::new(move |_| {
                                                if matches!(repo_resource.get_untracked(), Some(Some(_))) {
                                                    step.set(1);
                                                    error.set(None);
                                                }
                                            })
                                            attr:data-testid="help-report-no-account"
                                        >
                                            "I don't have an account"
                                        </Button>
                                        <Button appearance=ButtonAppearance::Primary on_click=close>
                                            "Close"
                                        </Button>
                                    }.into_any()
                                } else {
                                    let route = location.pathname.get();
                                    view! {
                                        <Button
                                            appearance=ButtonAppearance::Secondary
                                            on_click=Callback::new(move |_| step.set(0))
                                        >
                                            "Back"
                                        </Button>
                                        <Button
                                            appearance=ButtonAppearance::Primary
                                            disabled=Signal::derive(move || {
                                                submitting.get()
                                                    || !matches!(repo_resource.get(), Some(Some(_)))
                                            })
                                            on_click=Callback::new(move |_| {
                                                if submitting.get_untracked() {
                                                    return;
                                                }
                                                if !matches!(repo_resource.get_untracked(), Some(Some(_))) {
                                                    return;
                                                }
                                                submitting.set(true);
                                                error.set(None);
                                                let route = route.clone();
                                                leptos::task::spawn_local(async move {
                                                    let result = match kind {
                                                        HelpReportKind::Bug => {
                                                            submit_help_bug_report(
                                                                route,
                                                                title.get_untracked(),
                                                                description.get_untracked(),
                                                                steps_to_repro.get_untracked(),
                                                                expected.get_untracked(),
                                                                actual.get_untracked(),
                                                                nonempty(app_version.get_untracked()),
                                                                nonempty(browser_os.get_untracked()),
                                                                nonempty(contact.get_untracked()),
                                                            )
                                                            .await
                                                            .map(Some)
                                                        }
                                                        HelpReportKind::Feature => {
                                                            submit_help_feature_request(
                                                                route,
                                                                title.get_untracked(),
                                                                problem.get_untracked(),
                                                                proposed.get_untracked(),
                                                                nonempty(alternatives.get_untracked()),
                                                                nonempty(contact.get_untracked()),
                                                            )
                                                            .await
                                                            .map(Some)
                                                        }
                                                        HelpReportKind::Security => {
                                                            submit_help_security_report(
                                                                route,
                                                                summary.get_untracked(),
                                                                description.get_untracked(),
                                                                repro.get_untracked(),
                                                                affected.get_untracked(),
                                                                nonempty(severity.get_untracked()),
                                                                nonempty(contact.get_untracked()),
                                                            )
                                                            .await
                                                            .map(|()| None)
                                                        }
                                                    };
                                                    submitting.set(false);
                                                    match result {
                                                        Ok(Some(url)) => {
                                                            success.set(Some(format!("Filed: {url}")));
                                                        }
                                                        Ok(None) => {
                                                            success.set(Some(
                                                                "Thanks. Your security report was submitted privately.".into(),
                                                            ));
                                                        }
                                                        Err(e) => error.set(Some(e.to_string())),
                                                    }
                                                });
                                            })
                                            attr:data-testid="help-report-submit"
                                        >
                                            "Submit"
                                        </Button>
                                    }.into_any()
                                }
                            }}
                        </DialogActions>
                    </DialogBody>
                </div>
            </DialogSurface>
        </Dialog>
    }
}

fn nonempty(s: String) -> Option<String> {
    let t = s.trim().to_string();
    if t.is_empty() {
        None
    } else {
        Some(t)
    }
}
