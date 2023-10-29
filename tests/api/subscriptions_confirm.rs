use url::Url;
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
    let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

    let get_link = |s: &str| {
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect();
        assert_eq!(links.len(), 1);

        links[0].as_str().to_owned()
    };

    let raw_link = get_link(&body["HtmlBody"].as_str().unwrap());
    let mut confirmation_link = Url::parse(&raw_link).unwrap();
    assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");

    // Because of the way our test framework is set up, our fake base URL doesn't
    // specify the port. We need to inject that now, before sending the request.
    confirmation_link
        .set_port(Some(app.port))
        .expect("Failed to modify port");
    let response = reqwest::get(confirmation_link)
        .await
        .expect("Failed to execute confirmation request");

    assert_eq!(response.status().as_u16(), 200);
}
