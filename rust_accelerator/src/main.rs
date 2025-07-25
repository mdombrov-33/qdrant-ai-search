// Bring in necessary parts of actix-web
use actix_web::{App, HttpServer, Responder, get};

// Define GET /ping route with an async handler
#[get("/ping")]
async fn ping() -> impl Responder {
    // Return a plain string as the HTTP response
    "pong"
}

// Entry point to the app, async because actix needs async
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Log startup message
    println!("Starting Rust microservice on http://0.0.0.0:5000");

    // Start HTTP server
    HttpServer::new(|| {
        // For each connection, spin up a new App with routes
        App::new().service(ping) // Register the /ping route
    })
    .bind("0.0.0.0:5000")? // Bind to all interfaces on port 5000
    .run() // Start the server
    .await?; // Wait for server to finish (never, unless killed), propagate error if any

    println!("Server stopped gracefully");
    Ok(())
}
