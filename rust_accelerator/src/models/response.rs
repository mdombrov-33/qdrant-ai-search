use super::request::Metadata; // Import the Metadata struct from the request module
use serde::Deserialize; // Allows this struct to be built from JSON

/// A single result item returned *after* the Rust service re-ranks it
#[derive(Debug, Deserialize)]
pub struct ReRankedResult {
    /// Unique identifier for the document chunk
    pub id: String,

    /// Text content of the result (e.g., a chunk of a document)
    pub text: String,

    /// Final adjusted relevance score after processing
    pub score: f32,

    /// Metadata about the document (file name, MIME type, etc.)
    pub metadata: Metadata,
}

/// The full response body returned from the /re-rank endpoint
#[derive(Debug, Deserialize)]
pub struct ReRankResponse {
    /// List of top re-ranked results returned to FastAPI
    pub results: Vec<ReRankedResult>,

    /// Time in milliseconds it took to process the results
    pub processing_time_ms: u64,
}
