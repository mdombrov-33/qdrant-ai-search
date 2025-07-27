//! Request models for the re-ranking API.
//!
//! These structures define the shape of data coming into our service.
//! They correspond to the JSON that FastAPI sends us.

use serde::{Deserialize, Serialize};

/// The main request structure for the /re-rank endpoint.
///
/// This represents the JSON payload that FastAPI sends to our Rust service.
/// The `Deserialize` trait allows us to automatically convert JSON to this struct.
///
/// In TypeScript, this would be an interface:
/// ```typescript
/// interface ReRankRequest {
///   query: string;
///   results: SearchResult[];
///   limit: number;
/// }
/// ```
#[derive(Debug, Deserialize)]
pub struct ReRankRequest {
    /// The original search query from the user
    pub query: String,

    /// Raw search results from Qdrant that need re-ranking
    pub results: Vec<SearchResult>,

    /// Maximum number of results to return after re-ranking
    pub limit: usize,
}

/// A single search result from Qdrant.
///
/// This represents one document chunk with its similarity score and metadata.
/// The `Clone` trait allows us to make copies when needed.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchResult {
    /// Unique identifier for this document chunk
    pub id: String,

    /// The actual text content of this result
    pub text: String,

    /// Similarity score from Qdrant (0.0 to 1.0)
    pub score: f64,

    /// Additional information about this result
    pub metadata: ResultMetadata,
}

/// Metadata associated with a search result.
///
/// This contains extra information about where the result came from
/// and what type of content it is.
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
