use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use wiremock::MockServer;
use zero2prod::{
    configuration::{get_configuration, DatabaseSettings},
    startup::{get_connection_pool, Application},
    telemetry::{get_subscriber, init_subscriber},
};

// Ensure that we only initialize our subscriber once by wrapping in Lazy
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "debug".into();
    let subscriber_name = "test".into();

    // We use an environment variable to decide whether to swallow logs.
    // Need two separate blocks because the generic types on get_subscriber differ
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

/// Description of a mock app spun up for integration testing
pub struct TestApp {
    /// Address to send requests to the mock app
    pub address: String,
    /// Pool to use for DB connections in testing
    pub db_pool: PgPool,
    pub email_server: MockServer,
}

impl TestApp {
    /// Send a POST with `body` to the subscriptions API of our mocked app
    pub async fn post_subscriptions(
        &self,
        body: String,
    ) -> Result<reqwest::Response, reqwest::Error> {
        reqwest::Client::new()
            .post(&format!("{}/subscribe", self.address))
            .header("Content-type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
    }

    /// Send a GET request to confirm a newsletter subscription
    pub async fn get_subscription_confirmation(&self) -> Result<reqwest::Response, reqwest::Error> {
        reqwest::Client::new()
            .get(&format!("{}/subscriptions/confirm", &self.address))
            .send()
            .await
    }

    /// Send a GET to the health_check API of our mocked app
    pub async fn get_health_check(&self) -> Result<reqwest::Response, reqwest::Error> {
        reqwest::Client::new()
            .get(&format!("{}/health_check", &self.address))
            .send()
            .await
    }
}

/// Spins up a testing app to write integration tests against.
/// Returns the address to connect to.
pub async fn spawn_app() -> TestApp {
    // TRACING will only run the first time this function is called.
    Lazy::force(&TRACING);

    // Stand in for the Postmark email API
    let email_server = MockServer::start().await;

    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration");
        // Randomize DB name to avoid collisions between tests
        c.database.database_name = Uuid::new_v4().to_string();
        // Ask the OS for a random port
        c.application.port = 0;
        c.email_client.base_url = email_server.uri();

        c
    };
    // We create a new DB on each test case run, need to handle that now
    configure_database(&configuration.database).await;

    let app = Application::build(configuration.clone())
        .await
        .expect("Failed to build application");
    let address = format!("http://127.0.0.1:{}", app.port());
    let _ = tokio::spawn(app.run_until_stopped());

    TestApp {
        address,
        db_pool: get_connection_pool(&configuration.database),
        email_server,
    }
}

async fn configure_database(config: &DatabaseSettings) {
    // Build a new DB from scratch, and run all of our migrations

    let mut connection = PgConnection::connect_with(&config.instance_connection())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database");

    let connection_pool = PgPool::connect_with(config.database_connection())
        .await
        .expect("Failed to connect to database");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");
}
