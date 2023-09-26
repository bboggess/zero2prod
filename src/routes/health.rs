use actix_web::{get, HttpResponse};

/// Health check endpoint that will always respond with a 200 response
#[get("/health_check")]
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}
