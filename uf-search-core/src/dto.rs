//! Client-safe search source DTOs (no SSR / Valence dependency).

use serde::{Deserialize, Serialize};

/// Identifies one registered search source (id + display label), as shown to the client
/// for source selection (e.g. checkboxes in a search-scope picker).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SearchSourceKey {
    /// Stable identifier, matched against `SearchSourceDescriptor::id` server-side (`ssr`).
    pub id: String,
    /// Human-readable label for display in source-selection UI.
    pub label: String,
}

impl SearchSourceKey {
    /// Construct a key from an id and display label.
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }
}

/// A single search result row, as returned to the client.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchSourceItem {
    /// Which registered source produced this result (matches [`SearchSourceKey::id`]).
    pub source_id: String,
    /// Result identifier, meaningful within `source_id` (e.g. a record id or route).
    pub id: String,
    /// Display title for the result row.
    pub title: String,
    /// Optional secondary text (e.g. a snippet or subtitle).
    pub description: Option<String>,
    /// Free-form result kind/category (e.g. `"app"`, `"notification"`), used for icon/grouping.
    pub kind: String,
    /// Optional image URL the caller may show, such as a profile photo. Sources decide
    /// what they expose; absent in payloads from sources that predate this field.
    #[serde(default)]
    pub image_url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const LEGACY_JSON: &str =
        r#"{"source_id":"apps","id":"a","title":"A","description":null,"kind":"app"}"#;

    #[test]
    fn item_without_image_url_deserializes_happy_path() {
        let item: SearchSourceItem = serde_json::from_str(LEGACY_JSON).unwrap();
        assert_eq!(item.image_url, None);
        assert_eq!(item.title, "A");
    }

    #[test]
    fn item_with_image_url_round_trips_happy_path() {
        let item = SearchSourceItem {
            image_url: Some("https://example.test/a.png".into()),
            ..Default::default()
        };
        let back: SearchSourceItem =
            serde_json::from_str(&serde_json::to_string(&item).unwrap()).unwrap();
        assert_eq!(
            back.image_url.as_deref(),
            Some("https://example.test/a.png")
        );
    }

    #[test]
    fn item_with_null_image_url_is_none_sad_path() {
        let json = LEGACY_JSON.replace(r#""kind":"app""#, r#""kind":"app","image_url":null"#);
        let item: SearchSourceItem = serde_json::from_str(&json).unwrap();
        assert_eq!(item.image_url, None);
    }

    #[test]
    fn item_with_non_string_image_url_is_rejected_sad_path() {
        let json = LEGACY_JSON.replace(r#""kind":"app""#, r#""kind":"app","image_url":7"#);
        assert!(serde_json::from_str::<SearchSourceItem>(&json).is_err());
    }
}
