use std::net::TcpListener;

use actix_web::{dev::Server, get, App, HttpResponse, HttpServer, Responder};

/// Health check endpoint that will always respond with a 200 response
#[get("/health_check")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

/// Builds an HttpServer running our app at the specified address
pub fn run(listener: TcpListener) -> std::io::Result<Server> {
    let server = HttpServer::new(|| App::new().service(health_check))
        .listen(listener)?
        .run();

    Ok(server)
}
