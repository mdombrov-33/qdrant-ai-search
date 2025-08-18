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

// Prometheus counter for /re-rank requests
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

/// Application entry point.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    env_logger::init();

    println!(" Rust Accelerator - 2.4.0");

    // Initialize metrics counter
    let _counter = get_request_counter();
    println!(" Metrics initialized: rerank_requests_total counter ready");

    // Start HTTP server
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
