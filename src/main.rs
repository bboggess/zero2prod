use sqlx::PgPool;
use std::net::TcpListener;
use url::Url;
use zero2prod::configuration::get_configuration;
use zero2prod::email_client::EmailClient;
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration");

    let connection_pool = PgPool::connect_lazy_with(configuration.database.database_connection());

    let app_config = configuration.application;
    let app_address = format!("{}:{}", &app_config.host, app_config.port);
    let listener = TcpListener::bind(app_address)?;

    let email_config = configuration.email_client;
    let base_url = Url::parse(&email_config.base_url).expect("Invalid base URL");
    let sender_email = email_config.sender().expect("Invalid sender email address");
    let timeout = email_config.timeout();
    let email_client = EmailClient::new(
        base_url,
        sender_email,
        email_config.authorization_token,
        timeout,
    );

    run(listener, connection_pool, email_client)?.await
}
