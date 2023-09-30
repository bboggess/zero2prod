use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use sqlx::postgres::PgConnectOptions;

/// App-wide configuration
#[derive(Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
}

#[derive(Deserialize)]
pub struct ApplicationSettings {
    pub host: String,
    pub port: u16,
}

/// Settings needed for connecting to a DB.
#[derive(Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

/// Reads app configuration from the default file location.
///
/// Returns an error if parsing the config file into a `Settings` struct fails. This
/// could be a problem reading from the file or a malformed file.
pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine current directory.");
    let config_dir = base_path.join("config");

    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT");

    config::Config::builder()
        .add_source(config::File::from(config_dir.join("base")).required(true))
        .add_source(config::File::from(config_dir.join(environment.as_str())).required(true))
        .add_source(config::Environment::with_prefix("app").separator("__"))
        .build()?
        .try_deserialize()
}

pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            s => Err(format!(
                "{} is not a supported environment. Use either `local` or `production`",
                s
            )),
        }
    }
}

impl DatabaseSettings {
    /// Get a string to connect to a specific DB
    pub fn database_connection(&self) -> PgConnectOptions {
        self.instance_connection().database(&self.database_name)
    }

    /// Get a string to connect to an instance, for doing work unrelated to a specific
    /// DB. E.g. if we need to create new DB.
    pub fn instance_connection(&self) -> PgConnectOptions {
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(&self.password.expose_secret())
            .port(self.port)
    }
}
