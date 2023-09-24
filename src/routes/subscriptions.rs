use actix_web::{post, web, HttpResponse, Responder};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

/// The data being submitted from the subscription form
#[derive(Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

/// Adds a new subscription.
#[post("/subscribe")]
pub async fn subscribe(form: web::Form<FormData>, pool: web::Data<PgPool>) -> impl Responder {
    let query_result = sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(pool.get_ref())
    .await;

    match query_result {
        Ok(_) => HttpResponse::Ok(),
        Err(err) => {
            println!("Failed to execute query: {}", err);
            HttpResponse::InternalServerError()
        }
    }
}
