//! Host-provided callback to prefetch WASM chunks for an app family on hover.

use std::sync::Arc;

/// Callback type: receives an app `route_path` such as `"/counter"`.
pub type PrefetchFn = Arc<dyn Fn(&str) + Send + Sync>;

/// Provided by the host so the apps directory can warm lazy WASM chunks.
#[derive(Clone)]
pub struct PrefetchAppFamily(pub PrefetchFn);
