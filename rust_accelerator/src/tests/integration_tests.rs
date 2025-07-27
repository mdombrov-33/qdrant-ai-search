//! Integration tests for the HTTP API.
//!
//! These tests verify that the entire HTTP pipeline works correctly,
//! from receiving JSON requests to returning JSON responses.

use actix_web::{App, test, web};
use rust_accelerator::handlers;
use rust_accelerator::models::request::{ReRankRequest, ResultMetadata, SearchResult};
use serde_json::json;

/// Test the health check endpoint.
#[actix_web::test]
async fn test_health_check() {
    // Create test app with our routes
    let app = test::init_service(
        App::new().route("/health", web::get().to(handlers::health::health_check)),
    )
    .await;

    // Make request to health endpoint
    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;

    // Verify response
    assert!(resp.status().is_success());

    // Parse response body
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "healthy");
    assert_eq!(body["service"], "rust-accelerator");
}

/// Test the re-rank endpoint with valid input.
#[actix_web::test]
async fn test_rerank_valid_input() {
    let app = test::init_service(
        App::new()
            .route("/re-rank", web::post().to(handlers::rerank::handle_rerank))
            .app_data(web::JsonConfig::default().limit(1024 * 1024)),
    )
    .await;

    // Create test request payload
    let test_request = json!({
        "query": "machine learning algorithms",
        "results": [
            {
                "id": "doc1",
                "text": "Machine learning algorithms are powerful tools for data analysis and pattern recognition.",
                "score": 0.85,
                "metadata": {
                    "file_name": "ml_guide.pdf",
                    "content_type": "application/pdf"
                }
            },
            {
                "id": "doc2",
                "text": "The weather is nice today.",
                "score": 0.45,
                "metadata": {
                    "file_name": "weather.txt",
                    "content_type": "text/plain"
                }
            }
        ],
        "limit": 10,
        "threshold": 0.4
    });

    // Make request
    let req = test::TestRequest::post()
        .uri("/re-rank")
        .set_json(&test_request)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Verify response
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["results"].is_array());
    assert!(body["processing_time_ms"].is_number());
}

/// Test the re-rank endpoint with invalid input.
#[actix_web::test]
async fn test_rerank_invalid_input() {
    let app = test::init_service(
        App::new()
            .route("/re-rank", web::post().to(handlers::rerank::handle_rerank))
            .app_data(web::JsonConfig::default().limit(1024 * 1024)),
    )
    .await;

    // Test with empty query
    let test_request = json!({
        "query": "",
        "results": [],
        "limit": 10,
        "threshold": 0.5
    });

    let req = test::TestRequest::post()
        .uri("/re-rank")
        .set_json(&test_request)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Should return 400 Bad Request
    assert_eq!(resp.status(), 400);
}
