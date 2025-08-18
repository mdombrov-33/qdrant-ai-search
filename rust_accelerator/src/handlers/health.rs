//! Health check handler.

use actix_web::{HttpResponse, Result};
use serde_json::json;

/// Handles GET /health requests.
pub async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "status": "healthy",
        "service": "rust-accelerator",
        "version": env!("CARGO_PKG_VERSION"), // Automatically gets version from Cargo.toml
    })))
}
