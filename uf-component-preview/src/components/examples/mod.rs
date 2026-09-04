//! Example-only components that demonstrate catalog registration for product authors.

mod demo_status_pill;

pub use demo_status_pill::DemoStatusPill;

use uf_product::preview::PreviewRegistration;

#[cfg(feature = "preview")]
use demo_status_pill::DEMOSTATUSPILL_PREVIEW_REGISTRATION;

orbital_macros::preview_registrations! {
    &DEMOSTATUSPILL_PREVIEW_REGISTRATION,
}
