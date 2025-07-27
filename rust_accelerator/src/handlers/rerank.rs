use crate::models::{request::ReRankRequest, response::ReRankResponse};
use crate::services::rerank_service;
use actix_web::{HttpResponse, Result, post, web};

#[post("/re-rank")]
pub async fn rerank_handler(req_body: web::Json<ReRankRequest>) -> Result<HttpResponse> {
    // Call the service function
    let response: ReRankResponse = rerank_service::rerank_documents(req_body.into_inner()).await;

    Ok(HttpResponse::Ok().json(response))
}
