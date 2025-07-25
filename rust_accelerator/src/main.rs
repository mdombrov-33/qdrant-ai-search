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
    println!("Starting Rust microservice on http://127.0.0.1:5000");

    // Start HTTP server
    HttpServer::new(|| {
        // For each connection, spin up a new App with routes
        App::new().service(ping) // Register the /ping route
    })
    .bind("localhost:5000")? // Bind to localhost port 5000
    .run() // Start the server
    .await // Wait for server to finish (never, unless killed)
}
