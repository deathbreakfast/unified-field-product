//! Modal empty-state shell for auth / permission gates.

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use crate::components::{EmptyState, EmptyStateCallToAction};
use crate::primitives::{
    Button, ButtonAppearance, Dialog, DialogBody, DialogContent, DialogDismissConfig,
    DialogSurface, Material, MaterialCorners, MaterialElevation, MaterialVariant, OpenBind,
};

/// Whether [`crate::routes::RequireAuthenticated`] is showing an access gate
/// instead of page content.
///
/// Provided by [`provide_access_gate_state`] on the product shell. Help tours
/// read this so they do not auto-play over sign-in, email-verification, or
/// permission-required gates.
#[derive(Clone, Copy)]
pub struct AccessGateActive(RwSignal<bool>);

impl AccessGateActive {
    /// True while a RequireAuthenticated gate is showing instead of page content.
    #[must_use]
    pub fn get(&self) -> bool {
        self.0.get()
    }

    pub(crate) fn set(self, active: bool) {
        self.0.set(active);
    }
}

/// Publish whether an access gate (sign-in, email, or permission) is showing.
pub(crate) fn publish_access_gate(active: bool) {
    if let Some(gate) = use_access_gate_active() {
        gate.set(active);
    }
}

/// Provide [`AccessGateActive`] for the current component subtree.
///
/// Call once from the product shell so [`crate::routes::RequireAuthenticated`]
/// and Help share the flag.
pub fn provide_access_gate_state() -> AccessGateActive {
    let state = AccessGateActive(RwSignal::new(false));
    provide_context(state);
    state
}

/// Optional access to a provided [`AccessGateActive`].
#[must_use]
pub fn use_access_gate_active() -> Option<AccessGateActive> {
    use_context()
}

/// Modal shell for auth / permission empty-states so protected pages keep their
/// surrounding app bar chrome instead of rendering a blank full-page replacement.
///
/// Uses a narrow turf sheet for `DialogSurface` sizing (Orbital escape until
/// DialogSurface props cover this layout).
#[component]
pub(super) fn AccessGateDialog(
    /// Stable test id for the dialog root.
    #[prop(into)]
    test_id: String,
    /// Empty-state headline (the dialog's only title — no separate DialogTitle).
    message: &'static str,
    /// Empty-state supporting copy.
    description: &'static str,
    /// Illustration asset for the empty state.
    illustration_src: &'static str,
    /// Accessible alt text for the illustration.
    illustration_alt: &'static str,
    /// Primary / secondary CTAs besides Take me back.
    #[prop(optional)]
    children: Option<Children>,
) -> impl IntoView {
    let open = RwSignal::new(true);
    let dismissed = RwSignal::new(false);
    let navigate = use_navigate();
    // Yield to the app-bar AuthDialog so the two modals don't stack.
    if let Some(auth_dialog) = crate::use_auth_dialog_controller() {
        Effect::new(move |_| {
            if dismissed.get() {
                return;
            }
            open.set(!auth_dialog.open().get());
        });
    }
    let (style_sheet, class_names) = turf::inline_style_sheet_values! {
        .DialogFrame {
            background: transparent;
            border: none;
            padding: 0;
            width: fit-content;
            min-width: 280px;
            max-width: min(420px, calc(100vw - 48px));
        }

        .DialogMaterial {
            border-radius: var(--orb-radius-xl);
            padding: var(--spacingVerticalXXL);
            box-sizing: border-box;
            width: 100%;
        }
    };

    view! {
        <style>{style_sheet}</style>
        <div data-testid=test_id role="status" aria-live="polite">
            <Dialog
                open=OpenBind::from(open)
                dismiss=DialogDismissConfig {
                    mask_closeable: Signal::from(false),
                    close_on_esc: false,
                }
            >
                <DialogSurface class=class_names.dialog_frame>
                    <Material
                        class=class_names.dialog_material
                        variant=MaterialVariant::Frost
                        elevation=MaterialElevation::Modal
                        corners=MaterialCorners::Rounded
                    >
                        <DialogBody>
                            <DialogContent>
                                <EmptyState
                                    message=message
                                    description=description
                                    illustration_src=illustration_src
                                    illustration_alt=illustration_alt
                                >
                                    <EmptyStateCallToAction slot:call_to_action>
                                        {children.map(|c| c())}
                                        <Button
                                            appearance=ButtonAppearance::Subtle
                                            on_click=Callback::new({
                                                let navigate = navigate.clone();
                                                move |_| {
                                                    dismissed.set(true);
                                                    open.set(false);
                                                    crate::nav::navigate_back_or("/", &navigate);
                                                }
                                            })
                                        >
                                            "Take me back"
                                        </Button>
                                    </EmptyStateCallToAction>
                                </EmptyState>
                            </DialogContent>
                        </DialogBody>
                    </Material>
                </DialogSurface>
            </Dialog>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn access_gate_active_is_copy_happy_path() {
        fn assert_copy<T: Copy>() {}
        assert_copy::<AccessGateActive>();
    }
}
