//! Health check handler.
//!
//! Simple endpoint to verify the service is running and healthy.
//! This is commonly used by load balancers and monitoring systems.

use actix_web::{HttpResponse, Result};
use serde_json::json;

/// Handles GET /health requests.
///
/// This is a simple health check that returns a JSON response indicating
/// the service is running. In production, you might add more sophisticated
/// health checks (database connectivity, memory usage, etc.).
///
/// In Express.js, this would be:
/// ```javascript
/// app.get('/health', (req, res) => {
///   res.json({ status: 'healthy', service: 'rust-accelerator' });
/// });
/// ```
pub async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({
        "status": "healthy",
        "service": "rust-accelerator",
        "version": env!("CARGO_PKG_VERSION"), // Automatically gets version from Cargo.toml
    })))
}
