use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;

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
    config::Config::builder()
        .add_source(config::File::with_name("configuration"))
        .build()?
        .try_deserialize()
}

impl DatabaseSettings {
    /// Get a string to connect to a specific DB
    pub fn connection_string(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name
        ))
    }

    /// Get a string to connect to an instance, for doing work unrelated to a specific
    /// DB. E.g. if we need to create new DB.
    pub fn connection_string_for_instance(&self) -> Secret<String> {
        Secret::new(format!(
            "postgres://{}:{}@{}:{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port
        ))
    }
}
