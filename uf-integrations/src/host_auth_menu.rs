//! Host-provided auth menu for product shells that must not hard-depend on `lepton-shell`.

use std::sync::Arc;

use leptos::prelude::*;

/// Factory that renders the host's auth menu (typically `lepton_shell::AppBarUserMenu`).
#[derive(Clone)]
pub struct ShellAuthMenuFactory {
    render: Arc<dyn Fn() -> AnyView + Send + Sync>,
}

impl ShellAuthMenuFactory {
    /// Create a factory from a view builder.
    pub fn new<F, V>(f: F) -> Self
    where
        F: Fn() -> V + Send + Sync + 'static,
        V: IntoView,
    {
        Self {
            render: Arc::new(move || f().into_any()),
        }
    }

    /// Render the host auth menu.
    pub fn render(&self) -> AnyView {
        (self.render)()
    }
}

/// Provide the host auth menu for product layouts that call [`HostAuthMenu`].
pub fn provide_shell_auth_menu<F, V>(f: F)
where
    F: Fn() -> V + Send + Sync + 'static,
    V: IntoView,
{
    provide_context(ShellAuthMenuFactory::new(f));
}

/// Renders the host-provided auth menu, or nothing when the host did not call
/// [`provide_shell_auth_menu`].
#[component]
pub fn HostAuthMenu() -> impl IntoView {
    match use_context::<ShellAuthMenuFactory>() {
        Some(factory) => factory.render(),
        None => ().into_any(),
    }
}
