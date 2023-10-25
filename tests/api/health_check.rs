use crate::app;

#[actix_web::test]
async fn health_check_works() {
    let app = app::spawn_app().await;

    let response = app
        .get_health_check()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
