use crate::error::AppError;
use crate::models::response::ReRankedResult;
use crate::models::{request::ReRankRequest, response::ReRankResponse};
use crate::utils::timing::elapsed_ms;
use std::time::Instant;

pub async fn rerank_documents(req: ReRankRequest) -> Result<ReRankResponse, AppError> {
    let start = Instant::now();

    if req.query.trim().is_empty() {
        return Err(AppError::InvalidInput("Query string is empty".into()));
    }

    if req.results.is_empty() {
        return Err(AppError::InvalidInput("Results list is empty".into()));
    }

    let mut sorted_results = req.results;
    sorted_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    let re_ranked_results: Vec<ReRankedResult> = sorted_results
        .into_iter()
        .map(|item| ReRankedResult {
            id: item.id,
            text: item.text,
            score: item.score,
            metadata: item.metadata,
        })
        .collect();

    let processing_time_ms = elapsed_ms(start);

    Ok(ReRankResponse {
        results: re_ranked_results,
        processing_time_ms,
    })
}
