//! Optional host-supplied preview registrations merged into the catalog.

use std::sync::Mutex;

use uf_product::preview::PreviewRegistration;

static EXTENDED_REGISTRATIONS: Mutex<Vec<&'static PreviewRegistration>> = Mutex::new(Vec::new());

/// Merge additional preview registrations from zone crates or host wiring.
///
/// Call before the catalog serves `/orbital/{slug}` routes (typically in `main`
/// before Axum starts). Safe to call multiple times; each slice is appended in order.
pub fn extend_registrations(regs: &'static [&'static PreviewRegistration]) {
    if let Ok(mut guard) = EXTENDED_REGISTRATIONS.lock() {
        guard.extend_from_slice(regs);
    }
}

pub(crate) fn extended_registrations() -> Vec<&'static PreviewRegistration> {
    EXTENDED_REGISTRATIONS
        .lock()
        .map(|guard| guard.to_vec())
        .unwrap_or_default()
}
