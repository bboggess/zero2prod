use std::net::TcpListener;

use actix_web::{dev::Server, web, App, HttpServer};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;
use url::Url;

use crate::{
    configuration::Settings,
    email_client::EmailClient,
    routes::{health_check, subscribe},
};

/// Build an HTTP server running our app. The behavior of the app is configured
/// through the `settings` argument.
pub async fn build(settings: Settings) -> std::io::Result<Server> {
    let connection_pool = PgPool::connect_lazy_with(settings.database.database_connection());

    let email_config = settings.email_client;
    let base_url = Url::parse(&email_config.base_url).expect("Invalid base URL");
    let sender_email = email_config.sender().expect("Invalid sender email address");
    let timeout = email_config.timeout();
    let email_client = EmailClient::new(
        base_url,
        sender_email,
        email_config.authorization_token,
        timeout,
    );

    let app_config = settings.application;
    let app_address = format!("{}:{}", &app_config.host, app_config.port);
    let listener = TcpListener::bind(app_address)?;

    run(listener, connection_pool, email_client)
}

/// Starts a server, listening on `listener`, running in the background and returns it
pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
) -> std::io::Result<Server> {
    let db_pool = web::Data::new(db_pool);
    let email_client = web::Data::new(email_client);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .service(health_check)
            .service(subscribe)
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
