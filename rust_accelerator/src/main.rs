//! Main entry point for the Rust re-ranking service.
//!
//! This file sets up the HTTP server, configures logging, registers routes,
//! and starts the application. It's like the main() function in other languages
//! or the app setup in Express.js.

use actix_web::{App, HttpServer, middleware::Logger, web};

mod error;
mod handlers;
mod models;
mod services;
mod utils;

/// Main function - entry point of the application.
///
/// The `#[actix_web::main]` attribute sets up the async runtime.
/// This is similar to how Node.js handles async operations, but Rust
/// requires explicit async runtime setup.
///
/// In Node.js/Express, the equivalent would be:
/// ```javascript
/// const express = require('express');
/// const app = express();
///
/// app.use(express.json());
/// app.get('/health', healthHandler);
/// app.post('/re-rank', rerankHandler);
///
/// app.listen(5000, () => {
///   console.log('Server running on port 5000');
/// });
/// ```
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    // This sets up structured logging based on RUST_LOG environment variable
    // Set RUST_LOG=info for normal logging, RUST_LOG=debug for verbose logging
    env_logger::init();

    println!(" Rust Accelerator v2.4 - Balanced Domain Filtering");
    println!(" Build ID: domain-filter-balanced-v1");

    // Start HTTP server
    // Wrap bind in match to catch errors and log them gracefully
    let server = HttpServer::new(|| {
        App::new()
            // Add request logging middleware (like Morgan in Express)
            .wrap(Logger::default())
            // Register routes
            .route("/health", web::get().to(handlers::health::health_check))
            .route("/re-rank", web::post().to(handlers::rerank::handle_rerank))
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
