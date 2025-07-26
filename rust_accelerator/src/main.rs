use actix_web::{App, HttpServer, Responder, web};

async fn ping() -> impl Responder {
    "pong"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    eprintln!("🚀 Starting server...");
    eprintln!("🔧 Setting up HTTP server...");

    let result = HttpServer::new(|| {
        eprintln!("📦 Creating app instance...");
        App::new().route("/ping", web::get().to(ping))
    })
    .bind("0.0.0.0:5000");

    match result {
        Ok(server) => {
            eprintln!("✅ Successfully bound to 0.0.0.0:5000");
            eprintln!("🏃 Starting server run loop...");
            server.run().await
        }
        Err(e) => {
            eprintln!("❌ Failed to bind to 0.0.0.0:5000: {e}");
            Err(e)
        }
    }
}
