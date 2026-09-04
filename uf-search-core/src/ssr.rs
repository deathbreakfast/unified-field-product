//! SSR search-source provider trait, registry, and fan-out query helpers.

use super::{SearchSourceItem, SearchSourceKey};

/// Failure from a search-source provider or registry fan-out (`ssr`).
///
/// Converts from any `std::error::Error + Send + Sync + 'static` (including Valence
/// errors) so provider bodies can keep using `?`. Construct message-only failures with
/// [`Self::msg`]. [`SearchSourceRegistry::query_many`] attaches [`Self::source_id`] when
/// a registered descriptor fails.
///
/// This type intentionally does **not** implement `std::error::Error`, so the blanket
/// `From` conversion above stays coherent (same approach as `anyhow::Error`).
#[derive(Debug)]
pub struct SearchSourceError {
    source_id: Option<&'static str>,
    inner: anyhow::Error,
}

impl SearchSourceError {
    /// Build a message-only failure (no underlying `std::error::Error` source).
    #[must_use]
    pub fn msg(message: impl std::fmt::Display) -> Self {
        Self {
            source_id: None,
            inner: anyhow::anyhow!("{message}"),
        }
    }

    /// Registered descriptor id when the failure is attributed to one source.
    #[must_use]
    pub fn source_id(&self) -> Option<&'static str> {
        self.source_id
    }

    /// Attach the failing source id (used by registry fan-out).
    #[must_use]
    pub fn with_source_id(mut self, source_id: &'static str) -> Self {
        self.source_id = Some(source_id);
        self
    }
}

impl std::fmt::Display for SearchSourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.source_id {
            Some(id) => write!(f, "search source `{id}`: {}", self.inner),
            None => write!(f, "{}", self.inner),
        }
    }
}

impl<E> From<E> for SearchSourceError
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn from(error: E) -> Self {
        Self {
            source_id: None,
            inner: anyhow::Error::from(error),
        }
    }
}

/// Result type returned by [`SearchSourceProvider::query`].
pub type SearchSourceResult = Result<Vec<SearchSourceItem>, SearchSourceError>;

/// Boxed future returned by [`SearchSourceProvider::query`].
pub type SearchSourceFuture<'a> =
    std::pin::Pin<Box<dyn std::future::Future<Output = SearchSourceResult> + Send + 'a>>;

/// Implemented by each backend search source (one per registered [`SearchSourceDescriptor`]).
pub trait SearchSourceProvider: Send + Sync {
    /// Run a query against this source, returning at most `max_results` items.
    ///
    /// # Errors
    ///
    /// Return [`SearchSourceError`] when the Valence/query path fails (`?` from Valence
    /// errors works). [`SearchSourceRegistry::query_many`] propagates the first provider
    /// error (with `source_id` set) and stops fan-out.
    fn query<'a>(
        &'a self,
        valence: &'a valence::Valence,
        query_text: &'a str,
        max_results: u32,
    ) -> SearchSourceFuture<'a>;
}

/// Registration record for one backend search source, submitted via `inventory::submit!` and
/// collected into [`SearchSourceRegistry`] at first use.
pub struct SearchSourceDescriptor {
    /// Stable identifier matched against [`SearchSourceKey::id`] from client requests.
    pub id: &'static str,
    /// Human-readable label (mirrored to clients as [`SearchSourceKey::label`]).
    pub label: &'static str,
    /// Short description of what this source searches, for UI/help text.
    pub description: &'static str,
    /// The provider instance that actually executes queries for this source.
    pub provider: &'static dyn SearchSourceProvider,
}

quark::inventory::collect!(SearchSourceDescriptor);

impl quark::Registrable for SearchSourceDescriptor {
    fn registry_key(&self) -> &str {
        self.id
    }
}

// `quark::define_registry!` expands helper items without rustdoc.
#[allow(missing_docs)]
mod search_source_registry {
    use super::SearchSourceDescriptor;

    quark::define_registry! {
        /// Registry of all registered backend search sources.
        pub struct SearchSourceRegistry for SearchSourceDescriptor;
    }
}

pub use search_source_registry::SearchSourceRegistry;

impl SearchSourceRegistry {
    /// Resolve each requested [`SearchSourceKey`] to its registered descriptor, silently
    /// skipping keys with no matching registration (unknown ids do not error).
    pub fn list_descriptors_for_keys(
        &self,
        source_keys: &[SearchSourceKey],
    ) -> Vec<&'static SearchSourceDescriptor> {
        source_keys
            .iter()
            .filter_map(|source_key| self.get(source_key.id.as_str()))
            .collect()
    }

    /// Query every source in `source_keys` in turn, stopping once `max_results` items have
    /// been collected in total. Sources are queried in the order given. Unknown keys are
    /// skipped via [`Self::list_descriptors_for_keys`].
    ///
    /// # Errors
    ///
    /// Returns the first [`SearchSourceProvider::query`] error encountered, with
    /// [`SearchSourceError::source_id`] set to that descriptor's id. Earlier sources'
    /// items are discarded when a later source fails.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use uf_search_core::{SearchSourceKey, SearchSourceRegistry};
    ///
    /// async fn run(v: &valence::Valence) -> Result<(), uf_search_core::SearchSourceError> {
    ///     let registry = SearchSourceRegistry::auto_discover();
    ///     let keys = vec![SearchSourceKey::new("apps", "Apps")];
    ///     let hits = registry.query_many(&keys, v, "counter", 10).await?;
    ///     assert!(hits.len() <= 10);
    ///     Ok(())
    /// }
    /// ```
    pub async fn query_many(
        &self,
        source_keys: &[SearchSourceKey],
        valence: &valence::Valence,
        query_text: &str,
        max_results: u32,
    ) -> Result<Vec<SearchSourceItem>, SearchSourceError> {
        let descriptors = self.list_descriptors_for_keys(source_keys);
        let mut out = Vec::new();

        for descriptor in descriptors {
            if out.len() >= max_results as usize {
                break;
            }
            let remaining = (max_results as usize).saturating_sub(out.len()) as u32;
            let mut items = descriptor
                .provider
                .query(valence, query_text, remaining)
                .await
                .map_err(|err| err.with_source_id(descriptor.id))?;
            out.append(&mut items);
        }

        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_source_key_new_happy_path() {
        let key = SearchSourceKey::new("apps", "Apps");
        assert_eq!(key.id, "apps");
        assert_eq!(key.label, "Apps");
    }

    #[test]
    fn search_source_item_shape_happy_path() {
        let item = SearchSourceItem {
            source_id: "apps".into(),
            id: "apps".into(),
            title: "Apps".into(),
            description: Some("Apps directory".into()),
            kind: "app".into(),
        };
        assert_eq!(item.source_id, "apps");
        assert_eq!(item.kind, "app");
        assert_eq!(item.description.as_deref(), Some("Apps directory"));
    }

    #[test]
    fn search_source_item_missing_description_ok_sad() {
        let item = SearchSourceItem {
            source_id: "apps".into(),
            id: "x".into(),
            title: "X".into(),
            description: None,
            kind: "app".into(),
        };
        assert!(item.description.is_none());
    }
}

#[cfg(all(test, feature = "ssr"))]
mod registry_query_tests {
    use std::sync::Arc;

    use super::*;
    use valence::{InMemoryBackend, Valence};

    struct AlphaProvider;
    struct BetaProvider;
    struct FailingProvider;

    impl SearchSourceProvider for AlphaProvider {
        fn query<'a>(
            &'a self,
            _valence: &'a valence::Valence,
            _query_text: &'a str,
            max_results: u32,
        ) -> SearchSourceFuture<'a> {
            Box::pin(async move {
                let mut items = vec![SearchSourceItem {
                    source_id: "alpha".into(),
                    id: "a1".into(),
                    title: "Alpha One".into(),
                    description: None,
                    kind: "stub".into(),
                }];
                items.truncate(max_results as usize);
                Ok(items)
            })
        }
    }

    impl SearchSourceProvider for BetaProvider {
        fn query<'a>(
            &'a self,
            _valence: &'a valence::Valence,
            _query_text: &'a str,
            max_results: u32,
        ) -> SearchSourceFuture<'a> {
            Box::pin(async move {
                let mut items = vec![
                    SearchSourceItem {
                        source_id: "beta".into(),
                        id: "b1".into(),
                        title: "Beta One".into(),
                        description: None,
                        kind: "stub".into(),
                    },
                    SearchSourceItem {
                        source_id: "beta".into(),
                        id: "b2".into(),
                        title: "Beta Two".into(),
                        description: None,
                        kind: "stub".into(),
                    },
                ];
                items.truncate(max_results as usize);
                Ok(items)
            })
        }
    }

    impl SearchSourceProvider for FailingProvider {
        fn query<'a>(
            &'a self,
            _valence: &'a valence::Valence,
            _query_text: &'a str,
            _max_results: u32,
        ) -> SearchSourceFuture<'a> {
            Box::pin(async move { Err(SearchSourceError::msg("stub provider forced failure")) })
        }
    }

    static ALPHA: AlphaProvider = AlphaProvider;
    static BETA: BetaProvider = BetaProvider;
    static FAILING: FailingProvider = FailingProvider;

    static ALPHA_DESC: SearchSourceDescriptor = SearchSourceDescriptor {
        id: "alpha",
        label: "Alpha",
        description: "stub alpha",
        provider: &ALPHA,
    };
    static BETA_DESC: SearchSourceDescriptor = SearchSourceDescriptor {
        id: "beta",
        label: "Beta",
        description: "stub beta",
        provider: &BETA,
    };
    static FAILING_DESC: SearchSourceDescriptor = SearchSourceDescriptor {
        id: "failing",
        label: "Failing",
        description: "stub failing",
        provider: &FAILING,
    };

    fn mem_valence() -> Valence {
        Valence::builder()
            .add_backend("default", Arc::new(InMemoryBackend::new()))
            .build()
            .expect("in-memory valence")
    }

    fn registry_with(descs: &[&'static SearchSourceDescriptor]) -> SearchSourceRegistry {
        let mut registry = SearchSourceRegistry::new();
        for desc in descs {
            registry.register(*desc);
        }
        registry
    }

    #[tokio::test]
    async fn query_many_fans_out_in_key_order_and_caps_happy_path() {
        let valence = mem_valence();
        let registry = registry_with(&[&ALPHA_DESC, &BETA_DESC]);
        let keys = vec![
            SearchSourceKey::new("alpha", "Alpha"),
            SearchSourceKey::new("beta", "Beta"),
        ];

        let hits = registry
            .query_many(&keys, &valence, "", 10)
            .await
            .expect("query_many");
        assert_eq!(
            hits.iter().map(|h| h.title.as_str()).collect::<Vec<_>>(),
            ["Alpha One", "Beta One", "Beta Two"]
        );

        let capped = registry
            .query_many(&keys, &valence, "", 1)
            .await
            .expect("capped query_many");
        assert_eq!(capped.len(), 1);
        assert_eq!(capped[0].title, "Alpha One");
    }

    #[tokio::test]
    async fn list_descriptors_skips_unknown_keys_sad() {
        let registry = registry_with(&[&ALPHA_DESC]);
        let keys = vec![
            SearchSourceKey::new("zz-unknown", "Missing"),
            SearchSourceKey::new("alpha", "Alpha"),
        ];
        let descriptors = registry.list_descriptors_for_keys(&keys);
        assert_eq!(descriptors.len(), 1);
        assert_eq!(descriptors[0].id, "alpha");

        let valence = mem_valence();
        let hits = registry
            .query_many(
                &[SearchSourceKey::new("zz-unknown", "Missing")],
                &valence,
                "x",
                5,
            )
            .await
            .expect("unknown keys must not error");
        assert!(hits.is_empty());
    }

    #[tokio::test]
    async fn query_many_propagates_provider_error_sad() {
        let valence = mem_valence();
        let registry = registry_with(&[&ALPHA_DESC, &FAILING_DESC]);
        let keys = vec![
            SearchSourceKey::new("alpha", "Alpha"),
            SearchSourceKey::new("failing", "Failing"),
        ];
        let err = registry
            .query_many(&keys, &valence, "", 10)
            .await
            .expect_err("failing provider must surface Err");
        assert_eq!(err.source_id(), Some("failing"));
        assert!(
            err.to_string().contains("forced failure"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn search_source_error_msg_and_source_id_happy_path() {
        let err = SearchSourceError::msg("boom").with_source_id("apps");
        assert_eq!(err.source_id(), Some("apps"));
        assert!(err.to_string().contains("search source `apps`"));
        assert!(err.to_string().contains("boom"));
    }
}
