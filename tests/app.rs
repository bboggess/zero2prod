use std::net::TcpListener;

use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use zero2prod::configuration::{get_configuration, DatabaseSettings};

/// Description of a mock app spun up for integration testing
pub struct TestApp {
    /// Address to send requests to the mock app
    pub address: String,
    /// Pool to use for DB connections in testing
    pub db_pool: PgPool,
}

/// Spins up a testing app to write integration tests against.
/// Returns the address to connect to.
pub async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration = get_configuration().expect("Failed to read configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = configure_database(&configuration.database).await;

    let server = zero2prod::startup::run(listener, connection_pool.clone())
        .expect("Failed to bind to address");

    let _ = tokio::spawn(server);

    TestApp {
        address,
        db_pool: connection_pool,
    }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Build a new DB from scratch, and run all of our migrations

    let mut connection = PgConnection::connect(&config.connection_string_for_instance())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database");

    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to database");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}