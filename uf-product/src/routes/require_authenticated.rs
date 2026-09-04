//! Auth-aware route guards and access-gate dialog UI.

#[cfg(not(feature = "ssr"))]
use crate::permissions::check_permission_by_name;
#[cfg(feature = "ssr")]
use crate::permissions::eval_permission_by_name;
use crate::{AuthContext, AuthSession};
use leptos::prelude::*;
use leptos::tachys::view::any_view::{AnyView, IntoAny};
use leptos::task::spawn_local;
use leptos_router::hooks::{use_location, use_navigate};
use leptos_router::NavigateOptions;

use super::access_gate::{publish_access_gate, AccessGateDialog};
use super::referer::{auth_signin_href, auth_signup_href};
use crate::components::{EMPTYSTATE_LOCK_ILLUSTRATION, EMPTYSTATE_SIGNIN_ILLUSTRATION};
use crate::primitives::{Button, ButtonAppearance};

// Prefer direct backend eval on SSR (same request context as
// `provide_permission_backend`). Hydrate POSTs via `CheckPermissionByName`.
async fn has_permission_by_name(name: String) -> Result<bool, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        return eval_permission_by_name(&name).await;
    }
    #[cfg(not(feature = "ssr"))]
    {
        check_permission_by_name(name).await
    }
}

fn resolve_permission_id_by_name(
    _name: String,
) -> std::future::Ready<Result<Option<String>, ServerFnError>> {
    std::future::ready(Ok(None))
}

/// Fail-closed interpretation of a completed permission Resource result.
///
/// Only an explicit `Ok(Some(true))` grants access. `Ok(None)`, `Ok(Some(false))`,
/// and errors all deny.
fn permission_check_allows(result: &Result<Option<bool>, ServerFnError>) -> bool {
    matches!(result, Ok(Some(true)))
}

#[cfg(test)]
mod permission_gate_tests {
    //! Named gates must fail closed while Gauge is not wired as a git dep.

    use leptos::prelude::ServerFnError;

    #[test]
    fn permission_check_allows_only_explicit_true_happy_path() {
        assert!(super::permission_check_allows(&Ok(Some(true))));
    }

    #[test]
    fn permission_check_denies_none_false_and_err_sad() {
        assert!(!super::permission_check_allows(&Ok(None)));
        assert!(!super::permission_check_allows(&Ok(Some(false))));
        assert!(!super::permission_check_allows(&Err(ServerFnError::new(
            "check failed"
        ))));
    }
}

/// Build a condition closure suitable for auth-aware route guards.
///
/// This adapter converts the reactive [`AuthContext`] session signal into the
/// `Fn() -> Option<bool>` shape commonly expected by guard-style router components.
pub fn authenticated_route_condition(
    auth: &AuthContext,
) -> impl Fn() -> Option<bool> + Clone + 'static {
    let session = auth.session();
    move || match session.get() {
        AuthSession::Authenticated(_) => Some(true),
        AuthSession::Anonymous(_) => Some(false),
    }
}

fn current_path_with_search(pathname: &str, search: &str) -> String {
    let mut path = pathname.to_owned();
    if !search.is_empty() {
        path.push_str(search);
    }
    path
}

fn anonymous_gate(
    navigate: impl Fn(&str, NavigateOptions) + Clone + Send + Sync + 'static,
    auth_dialog: Option<crate::AuthDialogController>,
    location_pathname: Memo<String>,
    location_search: Memo<String>,
) -> AnyView {
    let nav_signin = navigate.clone();
    let nav_signup = navigate;
    let pathname_signin = location_pathname;
    let search_signin = location_search;
    let pathname_signup = location_pathname;
    let search_signup = location_search;
    view! {
        <AccessGateDialog
            test_id="auth-required-empty-state"
            message="Sign in required"
            description="Sign in or create an account to continue."
            illustration_src=EMPTYSTATE_SIGNIN_ILLUSTRATION
            illustration_alt="Sign in required"
        >
            <Button
                appearance=ButtonAppearance::Primary
                on_click=Callback::new(move |_| {
                    if let Some(dialog) = auth_dialog {
                        // Prefer the shell AuthDialog so the gated lazy route stays mounted.
                        dialog.open_signin();
                    } else {
                        let path = current_path_with_search(
                            &pathname_signin.get(),
                            &search_signin.get(),
                        );
                        nav_signin(
                            &auth_signin_href(Some(&path)),
                            NavigateOptions::default(),
                        );
                    }
                })
            >
                "Sign In"
            </Button>
            <Button
                appearance=ButtonAppearance::Subtle
                on_click=Callback::new(move |_| {
                    if let Some(dialog) = auth_dialog {
                        dialog.open_signup();
                    } else {
                        let path = current_path_with_search(
                            &pathname_signup.get(),
                            &search_signup.get(),
                        );
                        nav_signup(
                            &auth_signup_href(Some(&path)),
                            NavigateOptions::default(),
                        );
                    }
                })
            >
                "Sign Up"
            </Button>
        </AccessGateDialog>
    }
    .into_any()
}

fn email_verification_gate(
    navigate: impl Fn(&str, NavigateOptions) + Clone + Send + Sync + 'static,
) -> AnyView {
    view! {
        <AccessGateDialog
            test_id="email-verification-required-empty-state"
            message="Email verification required"
            description="Verify your email in account settings to continue."
            illustration_src=EMPTYSTATE_SIGNIN_ILLUSTRATION
            illustration_alt="Email verification required"
        >
            <Button
                appearance=ButtonAppearance::Primary
                on_click=Callback::new(move |_| {
                    navigate(crate::paths::USER_ACCOUNT_SETTINGS, NavigateOptions::default())
                })
            >
                "Account Settings"
            </Button>
        </AccessGateDialog>
    }
    .into_any()
}

fn permission_checking_gate() -> AnyView {
    view! {
        <AccessGateDialog
            test_id="permission-checking-empty-state"
            message="Checking permission"
            description="Verifying your access to this page."
            illustration_src=EMPTYSTATE_LOCK_ILLUSTRATION
            illustration_alt="Checking permission"
        />
    }
    .into_any()
}

fn permission_denied_gate(
    navigate: impl Fn(&str, NavigateOptions) + Clone + Send + Sync + 'static,
    permission_name: &'static str,
) -> AnyView {
    view! {
        <AccessGateDialog
            test_id="permission-required-empty-state"
            message="Permission required"
            description="You need additional access to view this content."
            illustration_src=EMPTYSTATE_LOCK_ILLUSTRATION
            illustration_alt="Permission required"
        >
            <Button
                appearance=ButtonAppearance::Primary
                on_click=Callback::new(move |_| {
                    let navigate = navigate.clone();
                    let permission_name = permission_name.to_string();
                    spawn_local(async move {
                        let _ = resolve_permission_id_by_name(permission_name).await;
                        navigate(crate::paths::PERMISSION_PERMISSIONS, NavigateOptions::default());
                    });
                })
            >
                "Request Permission"
            </Button>
        </AccessGateDialog>
    }
    .into_any()
}

/// Render children only when the current user satisfies the requested access rules.
///
/// [`RequireAuthenticated`] covers the three most common Orbital page gates:
///
/// - signed-in user only,
/// - signed-in user with verified email,
/// - signed-in user with a named permission.
///
/// When the requirement is not met, Orbital renders a guided empty-state instead of
/// a blank page. For example, anonymous users are prompted to sign in, while users
/// missing a permission get a shortcut into the permission-management UI.
///
/// Anonymous **Sign In** / **Sign Up** open the shell [`crate::AuthDialogController`]
/// when [`crate::provide_auth_dialog_controller`] ran under `UnifiedFieldShellLayout`
/// (and the host mounted `lepton_shell::AppBarUserMenu`). That keeps the current
/// lazy route mounted. Without a controller, the gate navigates to
/// [`crate::routes::auth_signin_href`] / [`crate::routes::auth_signup_href`] so
/// post-login can return via `?referer=`.
///
/// ## Examples
///
/// Basic authenticated page (pair with shell auth menu + dialog):
///
/// ```rust,ignore
/// // Host once: provide_shell_auth_menu(|| view! { <AppBarUserMenu /> });
/// view! {
///     <UnifiedFieldShellLayout>
///         <ShellAppBar slot>/* … HostAuthMenu … */</ShellAppBar>
///         <RequireAuthenticated>
///             <Dashboard />
///         </RequireAuthenticated>
///     </UnifiedFieldShellLayout>
/// }
/// ```
///
/// Require verified email:
///
/// ```rust,ignore
/// view! {
///     <RequireAuthenticated requires_email_verification=true>
///         <SensitiveSettingsPage />
///     </RequireAuthenticated>
/// }
/// ```
///
/// Require a platform permission:
///
/// ```rust,ignore
/// view! {
///     <RequireAuthenticated permission_name=Some("counter.admin.set")>
///         <CounterAdminPage />
///     </RequireAuthenticated>
/// }
/// ```
///
/// Named permission gates fail closed until Gauge is wired: only an explicit
/// allow from the permission check grants access; transport errors deny.
#[component]
pub fn RequireAuthenticated(
    /// Whether requires email verification is enabled.
    #[prop(optional, default = false)]
    requires_email_verification: bool,
    /// Optional permission name.
    #[prop(optional)]
    permission_name: Option<&'static str>,
    /// Child content rendered inside the component.
    children: ChildrenFn,
) -> impl IntoView {
    let auth = crate::use_auth_state();
    let navigate = use_navigate();
    let location = use_location();
    let location_pathname = location.pathname;
    let location_search = location.search;
    let auth_dialog = crate::use_auth_dialog_controller();
    let access_gate = super::access_gate::use_access_gate_active();
    let permission_check = Resource::new(
        move || auth.get(),
        move |session| async move {
            match (session, permission_name) {
                (AuthSession::Authenticated(_), Some(name)) => {
                    has_permission_by_name(name.to_string()).await.map(Some)
                }
                _ => Ok(None),
            }
        },
    );
    on_cleanup(move || {
        if let Some(gate) = access_gate {
            gate.set(false);
        }
    });

    view! {
        {move || {
            match auth.get() {
                AuthSession::Anonymous(_) => {
                    publish_access_gate(true);
                    anonymous_gate(
                        navigate.clone(),
                        auth_dialog,
                        location_pathname,
                        location_search,
                    )
                }
                AuthSession::Authenticated(user) => {
                    if requires_email_verification && !user.email_verified {
                        publish_access_gate(true);
                        email_verification_gate(navigate.clone())
                    } else if let Some(permission_name) = permission_name {
                        match permission_check.get() {
                            None => {
                                publish_access_gate(true);
                                permission_checking_gate()
                            }
                            Some(result) if permission_check_allows(&result) => {
                                publish_access_gate(false);
                                children().into_any()
                            }
                            Some(_) => {
                                publish_access_gate(true);
                                permission_denied_gate(navigate.clone(), permission_name)
                            }
                        }
                    } else {
                        publish_access_gate(false);
                        children().into_any()
                    }
                }
            }
        }}
    }
}
