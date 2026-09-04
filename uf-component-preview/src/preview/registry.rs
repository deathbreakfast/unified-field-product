use orbital_primitives::preview::PreviewCatalog;
use uf_product::preview::PreviewRegistration;

use super::extensions;
use super::manual_registrations;
use crate::components::examples;

/// Collect preview registrations from static tables only (SSR + WASM must match).
///
/// Merges zone-a / product locals, the Orbital leaf baseline, Tier C manuals, and
/// teaching examples via [`PreviewCatalog`] — product widgets stay in this workspace;
/// do not patch Orbital. Hosts may call [`extend_registrations`] to add zone pages.
pub fn collect_preview_registrations() -> Vec<&'static PreviewRegistration> {
    PreviewCatalog::new()
        .extend_many(uf_product::preview::collect_preview_registrations())
        .extend_many(orbital_primitives::preview::collect_all_preview_registrations())
        .extend(manual_registrations::all())
        .extend(examples::all())
        .extend_many(extensions::extended_registrations())
        .into_sorted_vec()
}
