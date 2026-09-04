//! Help service layer: visits, local mirror, pending helpers.

pub mod local_store;
pub mod visits;

pub use local_store::{
    local_mark_steps_seen, local_request_replay_for_route, read_local_visits,
    read_local_visits_for_route, write_local_visits, LOCAL_STORAGE_KEY,
};
pub use visits::{
    apply_replay_for_route, compute_pending, merge_local_into_server, replay_from_stored,
    replay_to_stored, HelpStepKey, HelpVisitRecord,
};
