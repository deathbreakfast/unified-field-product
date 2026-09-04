//! Shared bus so Help menu Replay can refresh [`HelpTourPlayer`].
//!
//! The app-bar Help control and the tour player are siblings under the shell, so
//! they cannot share Leptos context. A thread-local signal bridges them.

use std::cell::RefCell;

use leptos::prelude::*;

thread_local! {
    static HELP_REPLAY: RefCell<Option<RwSignal<u32>>> = const { RefCell::new(None) };
}

/// Install the replay tick signal (call once from [`super::HelpTourPlayer`]).
pub fn install_help_replay_bus(signal: RwSignal<u32>) {
    HELP_REPLAY.with(|cell| {
        *cell.borrow_mut() = Some(signal);
    });
}

/// Notify the tour player to reload visits after a replay request.
pub fn notify_help_replay() {
    HELP_REPLAY.with(|cell| {
        if let Some(signal) = *cell.borrow() {
            signal.update(|n| *n = n.wrapping_add(1));
        }
    });
}
