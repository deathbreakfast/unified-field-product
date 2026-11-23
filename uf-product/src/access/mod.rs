//! Shared views for who can act on one resource, behind a host-wired backend.
//!
//! A runtime app (Chronon jobs, Boson task types, Photon topics, Spectra tables) shows and
//! changes access to one resource with these views. The rules live in the host's
//! [`ResourceAccessBackend`], wired at boot like [`crate::permissions`]' `PermissionBackend`, so
//! no product crate depends on the permission store or on whoever manages teams.
//!
//! Hosts that wire nothing fail closed: [`require`] refuses every action, and the views say
//! "Access control isn't set up on this host".
//!
//! # Concern → API
//!
//! | Concern | API |
//! |---------|-----|
//! | Name a resource | [`ResourceRef`] |
//! | Roles shown instead of single actions | [`Role`] |
//! | Who holds what | [`AccessView`], [`AccessEntry`], [`PrincipalRef`] |
//! | Host implementation | [`ResourceAccessBackend`], [`provide_resource_access_backend`] |
//! | Gate a server function | [`require`] |
//! | Show and change access | [`AccessCard`] |
//! | Pick a team, group or person | [`PrincipalPicker`] |
//! | Stand in for what the reader can't use | [`AccessDenied`], [`DeniedActionButton`] |
//! | Ask the owners for access | [`RequestAccessDialog`] |
//!
//! # Wiring a backend
//!
//! The host implements the trait over its permission store and installs it with the rest of
//! the shell context, as the session actor's checks need the request context:
//!
//! ```rust,ignore
//! use std::sync::Arc;
//! use uf_product::access::{
//!     provide_resource_access_backend, AccessError, AccessView, PrincipalRef,
//!     PrincipalResults, RequestId, ResourceAccessBackend, ResourceRef, Role,
//! };
//!
//! struct StoreAccess;
//!
//! #[async_trait::async_trait]
//! impl ResourceAccessBackend for StoreAccess {
//!     async fn access(&self, r: &ResourceRef) -> Result<AccessView, AccessError> {
//!         todo!("read the resource's grants and owners as the session actor")
//!     }
//!     async fn set_role(&self, r: &ResourceRef, p: &PrincipalRef, role: Role) -> Result<(), AccessError> {
//!         todo!("replace p's grants on r in one write; refuse the last owner")
//!     }
//!     async fn remove(&self, r: &ResourceRef, p: &PrincipalRef) -> Result<(), AccessError> {
//!         todo!()
//!     }
//!     async fn search_principals(&self, q: &str) -> Result<PrincipalResults, AccessError> {
//!         todo!("teams, groups and people whose name matches q")
//!     }
//!     async fn request(&self, r: &ResourceRef, role: Role, reason: &str) -> Result<RequestId, AccessError> {
//!         todo!("send a request to r's owners")
//!     }
//!     async fn can(&self, r: &ResourceRef, action: &str) -> Result<bool, AccessError> {
//!         todo!()
//!     }
//! }
//!
//! // In the shell's context closure (routes and server functions):
//! provide_resource_access_backend(Arc::new(StoreAccess));
//! ```
//!
//! # Gating a server function
//!
//! ```rust,ignore
//! use leptos::prelude::*;
//! use uf_product::access::{require, ResourceRef};
//!
//! #[server]
//! pub async fn run_now(job: String) -> Result<(), ServerFnError> {
//!     let resource = ResourceRef::new("chronon_job", &job);
//!     require(&resource, "Run").await?; // session actor, fails closed
//!     // ... start the run as the same actor
//!     Ok(())
//! }
//! ```
//!
//! # Mounting the card
//!
//! ```rust,ignore
//! use leptos::prelude::*;
//! use uf_product::access::{AccessCard, AccessDenied, DeniedActionButton, ResourceRef};
//!
//! let job = ResourceRef::new("chronon_job", "billing-rollup")
//!     .label("Billing rollup")
//!     .noun("job");
//! view! {
//!     <AccessCard resource=job.clone() />
//!     // Where the reader lacks View, in place of the runs list only:
//!     <AccessDenied resource=job.clone() action="View" purpose="see this job's runs and logs" />
//!     // A button for an action the reader lacks:
//!     <DeniedActionButton resource=job action="Run" label="Run now" />
//! };
//! ```
//!
//! Role changes and adds ask for a recent authenticator code through the shell's step-up
//! dialog (the one `lepton-shell` mounts) and retry once the code is accepted.

mod access_card;
mod access_denied;
mod add_access_dialog;
mod backend;
mod parts;
mod principal_picker;
mod remove_access_dialog;
mod request_access_dialog;
mod server;
mod step_up;

#[cfg(all(test, feature = "ssr"))]
mod tests;

pub use access_card::{reader_notice, AccessCard, CardState};
pub use access_denied::{
    denied_sentence, denied_tooltip, AccessDenied, DeniedActionButton, DeniedInfo,
};
pub use add_access_dialog::{default_role, AddAccessDialog};
pub use backend::{
    is_last_owner, AccessEntry, AccessError, AccessLoad, AccessRequestReceipt, AccessView,
    PrincipalKind, PrincipalRef, PrincipalResults, RequestId, ResourceRef, Role, DENIED_PREFIX,
    LAST_OWNER_PREFIX, UNWIRED_PREFIX,
};
#[cfg(feature = "ssr")]
pub use backend::{
    provide_resource_access_backend, require, use_resource_access_backend, ResourceAccessBackend,
};
pub use principal_picker::{build_listing, PickerGroup, PickerListing, PickerRow, PrincipalPicker};
pub use remove_access_dialog::{write_error_text, RemoveAccessDialog};
pub use request_access_dialog::{goes_to_caption, RequestAccessDialog};
pub use server::{
    can_act_on_resource, load_resource_access, remove_resource_access, request_resource_access,
    search_access_principals, set_resource_access_role, MAX_REASON_CHARS, MIN_QUERY_CHARS,
};
pub use step_up::is_step_up_challenge;
