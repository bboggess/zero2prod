use std::net::TcpListener;

use actix_web::{dev::Server, App, HttpServer};

use crate::routes::health_check;

/// Builds an HttpServer running our app at the specified address
pub fn run(listener: TcpListener) -> std::io::Result<Server> {
    let server = HttpServer::new(|| App::new().service(health_check))
        .listen(listener)?
        .run();

    Ok(server)
}
