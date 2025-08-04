//! Main entry point for the Rust re-ranking service.
//!
//! This file sets up the HTTP server, configures logging, registers routes,
//! and starts the application.

use actix_web::{App, HttpResponse, HttpServer, Result, middleware::Logger, web};
use prometheus::{Counter, Encoder, TextEncoder, register_counter};
use std::sync::OnceLock;

mod error;
mod handlers;
mod models;
mod services;
mod utils;

#[cfg(test)]
mod tests;

// Global metrics - this will count total requests to /re-rank
static REQUEST_COUNTER: OnceLock<Counter> = OnceLock::new();

fn get_request_counter() -> &'static Counter {
    REQUEST_COUNTER.get_or_init(|| {
        register_counter!("rerank_requests_total", "Total number of re-rank requests")
            .expect("Failed to create counter")
    })
}

async fn metrics_handler() -> Result<HttpResponse> {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();

    Ok(HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4; charset=utf-8")
        .body(buffer))
}

/// Main function - entry point of the application.
///
/// The `#[actix_web::main]` attribute sets up the async runtime.
/// This is similar to how Node.js handles async operations, but Rust
/// requires explicit async runtime setup.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    // This sets up structured logging based on RUST_LOG environment variable
    // Set RUST_LOG=info for normal logging, RUST_LOG=debug for verbose logging
    env_logger::init();

    println!(" Rust Accelerator - 2.1.0");

    // Initialize metrics counter at startup so it appears in /metrics even with 0 requests
    let _counter = get_request_counter();
    println!(" Metrics initialized: rerank_requests_total counter ready");

    // Start HTTP server
    // Wrap bind in match to catch errors and log them gracefully
    let server = HttpServer::new(|| {
        App::new()
            // Add request logging middleware
            .wrap(Logger::default())
            // Register routes
            .route("/health", web::get().to(handlers::health::health_check))
            .route("/re-rank", web::post().to(handlers::rerank::handle_rerank))
            .route("/metrics", web::get().to(metrics_handler))
            // Set JSON payload size limit (default is 256KB, we increase it)
            .app_data(web::JsonConfig::default().limit(1024 * 1024)) // 1MB limit
    });

    match server.bind("0.0.0.0:5000") {
        Ok(server) => server.run().await,
        Err(e) => {
            eprintln!("Failed to bind server: {e}");
            Err(e)
        }
    }
}
