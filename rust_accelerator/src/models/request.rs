//! Request models for the re-ranking API.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The main request structure for the /re-rank endpoint.
#[derive(Debug, Deserialize)]
pub struct ReRankRequest {
    /// The original search query
    pub query: String,

    /// Raw search results to re-rank
    pub results: Vec<SearchResult>,

    /// Maximum number of results to return
    pub limit: usize,

    /// Optional IDF boost map
    #[serde(default)]
    pub idf_map: Option<HashMap<String, f64>>,

    pub threshold: f64,
}

/// A single search result from Qdrant.
///
/// This represents one document chunk with its similarity score and metadata.
/// The `Clone` trait allows us to make copies when needed.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchResult {
    /// Unique identifier for this document chunk
    pub id: String,

    /// Text content
    pub text: String,

    /// Similarity score (0.0 to 1.0)
    pub score: f64,

    /// Additional metadata
    pub metadata: ResultMetadata,
}

/// Metadata associated with a search result.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResultMetadata {
    /// Original filename of the document
    pub file_name: String,

    /// MIME type of the original document
    pub content_type: String,

    /// Optional: page number if from a PDF
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_number: Option<u32>,

    /// Optional: section or chapter title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section_title: Option<String>,
}
