use actix_web::{post, web, HttpResponse, Responder};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use tracing::Instrument;
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
    let email = &form.email;
    let name = &form.name;
    let request_id = Uuid::new_v4();

    let request_span = tracing::info_span!(
        "Adding a new subscriber.",
        %request_id,
        subscriber_email = %email,
        subscriber_name = %name
    );
    let _request_span_guard = request_span.enter();

    let query_span = tracing::info_span!("Saving subscriber details in database.");
    let query_result = sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        email,
        name,
        Utc::now()
    )
    .execute(pool.get_ref())
    .instrument(query_span)
    .await;

    match query_result {
        Ok(_) => HttpResponse::Ok(),
        Err(err) => {
            tracing::error!(
                "Request ID {}: Failed to execute query: {:?}",
                request_id,
                err
            );
            HttpResponse::InternalServerError()
        }
    }
}
