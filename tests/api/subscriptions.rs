use crate::app;
use sqlx;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

#[actix_web::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app = app::spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    let response = app.post_subscriptions(body.into()).await;

    assert_eq!(200, response.status().as_u16());
}

#[actix_web::test]
async fn subscribe_persists_new_subscriber() {
    let app = app::spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    let _ = app.post_subscriptions(body.into()).await;

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved info from the database");

    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.status, "pending_confirmation");
}

#[actix_web::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data() {
    let app = app::spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let _ = app.post_subscriptions(body.into()).await;

    // Mock::expect handles assertion that we sent POST to /email
}

#[actix_web::test]
async fn subscribe_sends_a_confirmation_email_with_a_link() {
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
    assert_eq!(confirmation_links.html, confirmation_links.html);
}

#[actix_web::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app = app::spawn_app().await;
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = app.post_subscriptions(invalid_body.into()).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with a 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

#[actix_web::test]
async fn subscribe_returns_a_400_when_fields_are_invalid() {
    let app = app::spawn_app().await;
    let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=definitely-not-an-email", "invalid email"),
    ];

    for (body, description) in test_cases {
        let response = app.post_subscriptions(body.into()).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with a 400 Bad Request when the payload was {}.",
            description
        );
    }
}
