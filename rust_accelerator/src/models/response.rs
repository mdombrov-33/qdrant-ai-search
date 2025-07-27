//! Response models for the re-ranking API.
//!
//! These structures define the shape of data we send back to FastAPI.
//! They correspond to the JSON response our service produces.

use super::request::ResultMetadata;
use serde::Serialize;

/// The main response structure for the /re-rank endpoint.
///
/// This represents the JSON we send back to FastAPI after processing.
/// The `Serialize` trait allows automatic conversion from struct to JSON.
#[derive(Debug, Serialize)]
pub struct ReRankResponse {
    /// Re-ranked and filtered results, limited to requested count
    pub results: Vec<ReRankedResult>,

    /// How long processing took in milliseconds
    pub processing_time_ms: u64,
}

/// A single re-ranked result.
///
/// This is similar to SearchResult but with an enhanced score
/// calculated by our algorithms.
#[derive(Debug, Serialize)]
pub struct ReRankedResult {
    /// Unique identifier (same as input)
    pub id: String,

    /// Text content (same as input)
    pub text: String,

    /// Enhanced score calculated by our algorithms
    pub score: f64,

    /// Metadata (same as input)
    pub metadata: ResultMetadata,
}
