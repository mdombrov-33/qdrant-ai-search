//! Response models for the re-ranking API.

use super::request::ResultMetadata;
use serde::Serialize;

/// The main response structure for the /re-rank endpoint.
#[derive(Debug, Serialize)]
pub struct ReRankResponse {
    /// Re-ranked and filtered results
    pub results: Vec<ReRankedResult>,

    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// A single re-ranked result.
#[derive(Debug, Serialize)]
pub struct ReRankedResult {
    /// Unique identifier
    pub id: String,

    /// Text content
    pub text: String,

    /// Enhanced score
    pub score: f64,

    /// Metadata
    pub metadata: ResultMetadata,
}
