//! Re-ranking request handler.
//!
//! This module handles the main /re-rank endpoint that receives search results
//! from FastAPI and returns optimized results.

use crate::error::AppError;
use crate::models::request::ReRankRequest;
use crate::services::rerank_service;
use actix_web::{HttpResponse, Result, web};
use log::{error, info};

/// Handles POST /re-rank requests.
///
/// This function:
/// 1. Receives and validates the JSON request
/// 2. Calls the re-ranking service
/// 3. Returns the enhanced results as JSON
///
/// The `web::Json<ReRankRequest>` parameter automatically deserializes
/// the incoming JSON into our Rust struct. If deserialization fails,
/// Actix-web automatically returns a 400 Bad Request error.
///
/// In Express.js with TypeScript, this would be:
/// ```typescript
/// app.post('/re-rank', async (req: Request<{}, ReRankResponse, ReRankRequest>, res) => {
///   try {
///     const result = await rerankDocuments(req.body);
///     res.json(result);
///   } catch (error) {
///     res.status(500).json({ error: error.message });
///   }
/// });
/// ```
pub async fn handle_rerank(req: web::Json<ReRankRequest>) -> Result<HttpResponse> {
    // Increment the request counter for Prometheus metrics
    crate::get_request_counter().inc();

    // Log the incoming request for debugging/monitoring
    info!(
        "Processing re-rank request with {} results",
        req.results.len()
    );

    // Call the main re-ranking service
    match rerank_service::rerank_documents(req.into_inner()).await {
        Ok(response) => {
            // Success: return the enhanced results
            info!("Re-ranking completed in {}ms", response.processing_time_ms);
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            // Error: log it and return appropriate HTTP status
            error!("Re-ranking failed: {e}",);

            // Convert our custom error to HTTP response
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
