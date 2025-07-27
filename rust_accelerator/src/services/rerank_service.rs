use crate::models::{request::ReRankRequest, response::ReRankResponse};

pub async fn rerank_documents(_req: ReRankRequest) -> ReRankResponse {
    ReRankResponse {
        results: vec![],
        processing_time_ms: 0,
    }
}
