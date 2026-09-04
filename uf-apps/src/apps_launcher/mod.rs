//! App-bar apps search launcher (Dialog + typeahead).
//!
//! Opens a centered search dialog over registered apps from `AppRegistry`.
//! Workspace principal pickers use `uf-integrations::SearchSourcePicker` instead.
//!
//! ## Concern → API
//!
//! | Concern | API |
//! |---------|-----|
//! | Dialog shell | [`AppsLauncher`] |
//! | Search + empty/results | [`AppsLauncherBody`] |
//! | Result row | [`AppsLauncherResult`] |
//! | Safe in-app navigate | [`safe_app_route_path`] |
//!
//! ## Typical flow
//!
//! ```rust,ignore
//! use leptos::prelude::*;
//! use uf_apps::apps_launcher::AppsLauncher;
//!
//! let open = RwSignal::new(false);
//! view! {
//!     <button on:click=move |_| open.set(true)>"Apps"</button>
//!     <AppsLauncher open=open />
//! };
//! ```

mod body;
mod launcher;
mod result_row;
mod safe_route;

pub use body::AppsLauncherBody;
pub use launcher::AppsLauncher;
pub use result_row::AppsLauncherResult;
pub use safe_route::safe_app_route_path;
