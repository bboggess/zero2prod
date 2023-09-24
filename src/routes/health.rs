use actix_web::{get, HttpResponse, Responder};

/// Health check endpoint that will always respond with a 200 response
#[get("/health_check")]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}
