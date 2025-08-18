//! Handler for the /re-rank endpoint.

use crate::error::AppError;
use crate::models::request::ReRankRequest;
use crate::services::rerank_service;
use actix_web::{HttpResponse, Result, web};
use log::{error, info};

/// Handles POST /re-rank requests.
///
/// Receives a JSON request, calls the re-ranking service, and returns the results as JSON.
pub async fn handle_rerank(req: web::Json<ReRankRequest>) -> Result<HttpResponse> {
    // Increment Prometheus counter
    crate::get_request_counter().inc();

    // Log incoming request
    info!(
        "Processing re-rank request with {} results",
        req.results.len()
    );

    // Call re-ranking service
    match rerank_service::rerank_documents(req.into_inner()).await {
        Ok(response) => {
            // Success: return results
            info!("Re-ranking completed in {}ms", response.processing_time_ms);
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            // Error: log and return HTTP status
            error!("Re-ranking failed: {e}",);

            // Convert error to HTTP response
            match e {
                AppError::InvalidInput(msg) => {
                    Ok(HttpResponse::BadRequest().json(serde_json::json!({
                        "error": "Invalid input",
                        "message": msg
                    })))
                }
            }
        }
    }
}
