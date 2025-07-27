pub mod handlers;
pub mod models;
pub mod services;
pub mod utils;

// Import required modules from the actix-web crate
// - App: to define routes
// - HttpServer: to run the server
// - web: contains helpers like route() and handler registration
use actix_web::{App, HttpServer, web};

// Import the specific handler function for health check
use handlers::{health::health_check, rerank::rerank_handler};

// The main entry point for our async Actix-web server
// #[actix_web::main] is a procedural macro that:
// - Sets up the async runtime (like tokio::main)
// - Prepares Actix's environment
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Print a message to stderr when the server starts
    // (This is just logging, not required for logic)
    eprintln!("Starting server...");

    // HttpServer::new is a function that creates a new server instance
    // - The `::` syntax means "access an associated function" (like static methods in Python/TS)
    // - This is like calling `HttpServer.new()` in OOP terms

    // The closure `|| App::new()` builds our app's routing configuration
    // - Each request spawns a new `App` instance
    // - .route() adds a GET endpoint at /health, using the health_check handler

    HttpServer::new(|| {
        App::new()
            .route("/health", web::get().to(health_check))
            .service(rerank_handler)
    })
    .bind("0.0.0.0:5000")? // Bind the server to listen on all interfaces (0.0.0.0) at port 5000
    .run() // This is important for Docker deployment — use 127.0.0.1 only for local dev
    .await // Start the server's event loop and wait for incoming requests
}
