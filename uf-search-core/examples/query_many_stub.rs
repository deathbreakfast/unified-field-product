//! Fan-out a search query across one in-memory stub provider.
//!
//! ## When to use
//! Prove `SearchSourceRegistry::query_many` without a Leptos host.
//!
//! ## Command
//! ```bash
//! cargo run -p uf-search-core --example query_many_stub --features ssr
//! ```
//!
//! ## Success
//! Stdout prints `query_many_stub: OK` with at least one hit titled `Beacon Alpha`.
//!
//! ## Look next
//! Wire results into `uf_integrations::SearchSourcePicker` (`shell-chrome-host`).

#![allow(clippy::print_stdout, clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use uf_search_core::{
    SearchSourceDescriptor, SearchSourceFuture, SearchSourceItem, SearchSourceKey,
    SearchSourceProvider, SearchSourceRegistry,
};
use valence::{InMemoryBackend, Valence};

struct StubBeaconProvider;

impl SearchSourceProvider for StubBeaconProvider {
    fn query<'a>(
        &'a self,
        _valence: &'a valence::Valence,
        query_text: &'a str,
        max_results: u32,
    ) -> SearchSourceFuture<'a> {
        Box::pin(async move {
            let needle = query_text.to_ascii_lowercase();
            let all = [
                SearchSourceItem {
                    source_id: "beacons".into(),
                    id: "alpha".into(),
                    title: "Beacon Alpha".into(),
                    description: Some("Teaching stub hit".into()),
                    kind: "beacon".into(),
                },
                SearchSourceItem {
                    source_id: "beacons".into(),
                    id: "beta".into(),
                    title: "Beacon Beta".into(),
                    description: None,
                    kind: "beacon".into(),
                },
            ];
            let hits: Vec<_> = all
                .into_iter()
                .filter(|item| {
                    needle.is_empty()
                        || item.title.to_ascii_lowercase().contains(&needle)
                        || item.id.to_ascii_lowercase().contains(&needle)
                })
                .take(max_results as usize)
                .collect();
            Ok(hits)
        })
    }
}

static PROVIDER: StubBeaconProvider = StubBeaconProvider;

quark::inventory::submit! {
    SearchSourceDescriptor {
        id: "beacons",
        label: "Beacons",
        description: "Stub beacon search source for the query_many example",
        provider: &PROVIDER,
    }
}

#[tokio::main]
async fn main() {
    let valence = Valence::builder()
        .add_backend("default", Arc::new(InMemoryBackend::new()))
        .build()
        .expect("valence mem backend");

    let registry = SearchSourceRegistry::auto_discover();
    let keys = vec![SearchSourceKey::new("beacons", "Beacons")];
    let hits = registry
        .query_many(&keys, &valence, "alpha", 10)
        .await
        .expect("query_many");

    assert!(
        hits.iter().any(|h| h.title == "Beacon Alpha"),
        "expected Beacon Alpha in {hits:?}"
    );
    println!(
        "query_many_stub: OK — {} hit(s), first={}",
        hits.len(),
        hits[0].title
    );
}
