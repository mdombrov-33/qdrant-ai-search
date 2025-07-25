use actix_web::{App, HttpServer, Responder, web};

async fn ping() -> impl Responder {
    "pong"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    eprintln!("🚀 Starting server...");

    HttpServer::new(|| App::new().route("/ping", web::get().to(ping)))
        .bind("0.0.0.0:5000")?
        .run()
        .await
}
