//! Spotlight tour player and replay bus.
//!
//! [`HelpTourPlayer`] is the stock mount point: it reads inventory for the current
//! pathname, resolves visit rows (Valence when signed in, [`crate::LOCAL_STORAGE_KEY`]
//! when signed out, merged on conflict), and opens Orbital `SpotlightTour` for pending steps. Auto-play is suppressed while [`uf_product::AccessGateActive`]
//! is set (sign-in, email verification, permission-required empty states).
//!
//! Replay is scoped to the active route: [`request_replay_current_route`] updates
//! local storage always and Valence when authenticated, then pings
//! [`notify_help_replay`] so the player reloads pending steps.

mod player;
mod replay_bus;

pub use player::{request_replay_current_route, HelpTourPlayer};
pub use replay_bus::notify_help_replay;
