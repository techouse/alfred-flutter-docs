use serde::Deserialize;

/// Minimal subset of an Algolia single-index search response.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct SearchResponse {
    /// Results in the provider's ranking order.
    pub hits: Vec<SearchResult>,
}

/// A Flutter API documentation result returned by Algolia.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct SearchResult {
    /// Algolia's stable object identifier.
    #[serde(rename = "objectID")]
    pub object_id: String,
    /// Result name displayed in Alfred.
    pub name: String,
    /// Fully qualified Dart name used as Alfred's large type text.
    #[serde(rename = "qualifiedName")]
    pub qualified_name: String,
    /// Relative Flutter API path returned by the index.
    pub href: String,
    /// Result kind (for example `class` or `method`).
    #[serde(rename = "type")]
    pub type_name: String,
    /// Enclosing declaration, when one exists.
    #[serde(rename = "enclosedBy")]
    pub enclosed_by: Option<EnclosedBy>,
}

/// Nullable enclosure metadata emitted by the Flutter index.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct EnclosedBy {
    /// Enclosing declaration name. Missing names retain Dart's `from null` behavior.
    pub name: Option<String>,
}
