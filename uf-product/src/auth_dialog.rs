//! Shared controller so app-bar auth UI and route gates open the same modal.
//!
//! `lepton-shell` and [`crate::routes::RequireAuthenticated`] bind to the same
//! signals when a controller is provided near the shell root.

use leptos::prelude::*;

/// Which auth dialog surface to show.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AuthDialogIntent {
    /// Email/password sign-in form.
    #[default]
    Signin,
    /// Create-account form.
    Signup,
    /// Confirm logout.
    Logout,
}

/// Reactive handle for opening the host auth dialog without a route change.
#[derive(Clone, Copy)]
pub struct AuthDialogController {
    open: RwSignal<bool>,
    intent: RwSignal<AuthDialogIntent>,
}

impl AuthDialogController {
    /// Create an unbound controller (not yet provided as context).
    #[must_use]
    pub fn new() -> Self {
        Self {
            open: RwSignal::new(false),
            intent: RwSignal::new(AuthDialogIntent::Signin),
        }
    }

    /// Whether the auth dialog should be visible.
    #[must_use]
    pub fn open(&self) -> RwSignal<bool> {
        self.open
    }

    /// Current dialog intent (sign-in / sign-up / logout).
    #[must_use]
    pub fn intent(&self) -> RwSignal<AuthDialogIntent> {
        self.intent
    }

    /// Show the sign-in form.
    pub fn open_signin(&self) {
        self.intent.set(AuthDialogIntent::Signin);
        self.open.set(true);
    }

    /// Show the sign-up form.
    pub fn open_signup(&self) {
        self.intent.set(AuthDialogIntent::Signup);
        self.open.set(true);
    }

    /// Show the logout confirmation.
    pub fn open_logout(&self) {
        self.intent.set(AuthDialogIntent::Logout);
        self.open.set(true);
    }

    /// Hide the auth dialog.
    pub fn close(&self) {
        self.open.set(false);
    }
}

impl Default for AuthDialogController {
    fn default() -> Self {
        Self::new()
    }
}

/// Provide [`AuthDialogController`] for the current component subtree.
pub fn provide_auth_dialog_controller() -> AuthDialogController {
    let controller = AuthDialogController::new();
    provide_context(controller);
    controller
}

/// Optional access to a provided [`AuthDialogController`].
#[must_use]
pub fn use_auth_dialog_controller() -> Option<AuthDialogController> {
    use_context::<AuthDialogController>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_dialog_intent_default_is_signin_happy_path() {
        assert_eq!(AuthDialogIntent::default(), AuthDialogIntent::Signin);
        assert_ne!(AuthDialogIntent::Signup, AuthDialogIntent::Logout);
    }

    #[test]
    fn auth_dialog_controller_is_copy_happy_path() {
        // Controllers are Copy handles; construction needs a reactive owner in UI,
        // so unit coverage sticks to the intent enum + type contracts.
        fn assert_copy<T: Copy>() {}
        assert_copy::<AuthDialogController>();
        assert_copy::<AuthDialogIntent>();
    }
}
