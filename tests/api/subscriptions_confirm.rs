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
