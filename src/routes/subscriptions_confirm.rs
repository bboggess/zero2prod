use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct Parameters {
    /// Require a token to confirm, otherwise will return an error status
    subscription_token: String,
}

/// Confirm a subscription. Afterwards, subscriber will start receiving the newsletter
#[tracing::instrument(name = "Confirming a pending subscription", skip(parameters))]
pub async fn confirm(parameters: web::Query<Parameters>, pool: web::Data<PgPool>) -> HttpResponse {
    let subscriber_id =
        match get_subscriber_id_from_token(&pool, &parameters.subscription_token).await {
            Ok(id) => id,
            Err(_) => return HttpResponse::InternalServerError().finish(),
        };

    let subscriber_id = match subscriber_id {
        Some(id) => id,
        None => return HttpResponse::Unauthorized().finish(),
    };

    if set_subscriber_confirmed(&pool, subscriber_id)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}

/// Looks up the subscriber to whom the `subscription_token` belongs. There may not
/// be a matching subscriber.
#[tracing::instrument(
    name = "Look up subscriber_id from token",
    skip(pool, subscription_token)
)]
async fn get_subscriber_id_from_token(
    pool: &PgPool,
    subscription_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    sqlx::query!(
        r#"SELECT subscriber_id FROM subscription_tokens
        WHERE subscription_token = $1"#,
        subscription_token
    )
    .fetch_optional(pool)
    .await
    .map_err(|err| {
        tracing::error!("Failed to execute query: {:?}", err);
        err
    })
    .map(|row| row.map(|r| r.subscriber_id))
}

/// Marks the subscriber with ID `subscriber_id` as 'confirmed' in the database.
#[tracing::instrument(name = "Mark subscriber as confirmed", skip(pool, subscriber_id))]
async fn set_subscriber_confirmed(pool: &PgPool, subscriber_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscriptions
        SET status = 'confirmed'
        WHERE id = $1"#,
        subscriber_id
    )
    .execute(pool)
    .await
    .map_err(|err| {
        tracing::error!("Failed to execute query: {:?}", err);
        err
    })?;

    Ok(())
}
