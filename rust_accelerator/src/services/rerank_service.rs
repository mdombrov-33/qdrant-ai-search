use crate::models::response::ReRankedResult;
use crate::models::{request::ReRankRequest, response::ReRankResponse};
use crate::utils::timing::elapsed_ms;
use std::time::Instant;

pub async fn rerank_documents(req: ReRankRequest) -> ReRankResponse {
    let start = Instant::now();

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

    ReRankResponse {
        results: re_ranked_results,
        processing_time_ms,
    }
}
