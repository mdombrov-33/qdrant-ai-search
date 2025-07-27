use super::request::Metadata;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ReRankedResult {
    pub id: String,
    pub text: String,
    pub score: f32,
    pub metadata: Metadata,
}

#[derive(Debug, Deserialize)]
pub struct ReRankResponse {
    pub results: Vec<ReRankedResult>,
    pub processing_time_ms: u64,
}
