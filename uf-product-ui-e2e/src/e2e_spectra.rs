//! Process-wide mem Spectra for Playwright usage-card scenarios.
//!
//! SpectraRouter::set_global is once-only — do not rebuild Spectra mid-process.
//! Scenario isolation uses distinct `usage_viewer` keys (see seed endpoint).

use std::sync::{Arc, OnceLock};

use spectra::{MemEventsBackend, MemMetricsBackend, Spectra};
use spectra_core::{EventStorageBackend, MetricsStorageBackend};

static E2E_SPECTRA: OnceLock<Arc<Spectra>> = OnceLock::new();

/// Shared mem Spectra for this e2e host process.
pub fn e2e_spectra() -> Arc<Spectra> {
    E2E_SPECTRA
        .get_or_init(|| {
            let metrics: Arc<dyn MetricsStorageBackend> = Arc::new(MemMetricsBackend::new());
            let events: Arc<dyn EventStorageBackend> = Arc::new(MemEventsBackend::new());
            Arc::new(
                Spectra::builder()
                    .metrics_backend(metrics)
                    .events_backend(events)
                    .embedded()
                    .build()
                    .expect("e2e mem Spectra"),
            )
        })
        .clone()
}
