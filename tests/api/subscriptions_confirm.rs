use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::app;

#[actix_web::test]
async fn confirmations_without_token_are_rejected_with_400() {
    let app = app::spawn_app().await;

    let response = app
        .get_subscription_confirmation()
        .await
        .expect("Failed to execute request");

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

    let _ = app
        .post_subscriptions(body.into())
        .await
        .expect("Failed to execute request");

    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(email_request);

    let response = reqwest::get(confirmation_links.html)
        .await
        .expect("Failed to execute confirmation request");

    assert_eq!(response.status().as_u16(), 200);
}
