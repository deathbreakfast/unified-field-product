//! Run an access write that may need a recent authenticator code.
//!
//! The first try runs straight away. When the server asks for a code, the shell's step-up
//! dialog opens, and the write runs once more after the code is accepted. A host whose shell
//! provides no step-up controller shows the server's message instead.

use lepton_auth::actions::step_up::verify_step_up_totp;
use lepton_auth_ui::{
    use_step_up_controller, StepUpController, StepUpFactors, StepUpPolicy, StepUpRequest,
};
use leptos::prelude::*;
use leptos::task::spawn_local_scoped;

/// True when the server asked for (or expired) a recent authenticator code.
#[must_use]
pub fn is_step_up_challenge(err: &impl std::fmt::Display) -> bool {
    let text = err.to_string();
    text.contains("STEP_UP:step_up_required") || text.contains("STEP_UP:step_up_expired")
}

fn window_request() -> StepUpRequest {
    StepUpRequest {
        title: "Confirm it's you".to_string(),
        description: Some(
            "Enter the code from your authenticator app to change who has access.".to_string(),
        ),
        policy: StepUpPolicy::Totp,
    }
}

/// Run `action`; on a code challenge, open step-up, verify, then retry once.
///
/// `on_ok` runs after success. `error` gets the server's message when the call fails for
/// another reason, or stays set when the person closes the dialog without a code.
pub fn spawn_with_step_up<F, Fut>(error: RwSignal<Option<String>>, on_ok: Callback<()>, action: F)
where
    F: Fn() -> Fut + Clone + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<(), ServerFnError>> + 'static,
{
    let step_up = use_step_up_controller();
    spawn_local_scoped(async move {
        match action().await {
            Ok(()) => on_ok.run(()),
            Err(err) if is_step_up_challenge(&err) => {
                // Shown if the dialog is closed without confirming.
                error.set(Some(err.to_string()));
                if let Some(ctrl) = step_up {
                    request_code_then_retry(ctrl, error, on_ok, action);
                }
            }
            Err(err) => error.set(Some(err.to_string())),
        }
    });
}

fn request_code_then_retry<F, Fut>(
    ctrl: StepUpController,
    error: RwSignal<Option<String>>,
    on_ok: Callback<()>,
    action: F,
) where
    F: Fn() -> Fut + Clone + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<(), ServerFnError>> + 'static,
{
    ctrl.request(
        window_request(),
        Callback::new(move |factors: StepUpFactors| {
            let action = action.clone();
            spawn_local_scoped(async move {
                if let Err(e) = verify_step_up_totp(factors.totp_code).await {
                    ctrl.report_error(e.to_string());
                    return;
                }
                match action().await {
                    Ok(()) => {
                        ctrl.complete_success();
                        error.set(None);
                        on_ok.run(());
                    }
                    Err(e) if is_step_up_challenge(&e) => ctrl.report_error(e.to_string()),
                    Err(e) => {
                        ctrl.complete_success();
                        error.set(Some(e.to_string()));
                    }
                }
            });
        }),
    );
}
