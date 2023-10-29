use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::app;

#[actix_web::test]
async fn confirmations_without_token_are_rejected_with_400() {
    let app = app::spawn_app().await;

    let response = app.get_subscription_confirmation().await;

    assert_eq!(response.status(), 400);
}

#[actix_web::test]
async fn the_link_returned_by_subscribe_returns_200_when_called() {
    let app = app::spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    let _ = app.post_subscriptions(body.into()).await;

    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(email_request);

    let response = reqwest::get(confirmation_links.html)
        .await
        .expect("Failed to execute confirmation request");

    assert_eq!(response.status().as_u16(), 200);
}

#[actix_web::test]
async fn clicking_on_confirmation_link_confirms_a_subscriber() {
    let app = app::spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    let _ = app.post_subscriptions(body.into()).await;

    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(email_request);

    reqwest::get(confirmation_links.html)
        .await
        .expect("Failed to execute confirmation request");

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved info from the database");

    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.status, "confirmed");
}
