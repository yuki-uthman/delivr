use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;
use std::convert::{TryFrom, TryInto};

use crate::error::{Error, Result};

#[derive(serde::Deserialize, Clone, Debug)]
pub struct Config {
    pub application: Application,
    pub environment: Environment,
    pub database: Database,
    pub zoho: Zoho,
}

impl Config {
    pub fn addr(&self) -> String {
        format!("{}:{}", self.application.host, self.application.port)
    }
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct Application {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub base_url: String,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct Database {
    pub host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub username: String,
    pub password: Secret<String>,
    pub database_name: String,
    pub require_ssl: bool,
}

impl Database {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name
        )
    }

    /// Omitting the database name connects to the Postgres instance, not a specific logical database.
    /// This is useful for operations that create or drop databases.
    pub fn connection_string_without_db(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port
        )
    }
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct Zoho {
    pub client_id: String,
    pub client_secret: Secret<String>,
}

pub fn get_config() -> Result<Config> {
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .map_err(Error::from)?;

    if environment == Environment::Local || environment == Environment::Test {
        dotenvy::dotenv().ok();
    }

    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("configuration");
    let environment_filename = format!("{}.yaml", environment.as_str());

    let config = config::Config::builder()
        .add_source(config::File::from(
            configuration_directory.join("base.yaml"),
        ))
        .add_source(config::File::from(
            configuration_directory.join(environment_filename),
        ))
        // Add in settings from environment variables (with a prefix of APP and '__' as separator)
        // E.g. `APP_APPLICATION__PORT=5001 would set `Settings.application.port`
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;

    let mut config = config.try_deserialize::<Config>().map_err(Error::from)?;
    config.environment = environment;

    Ok(config)
}

/// The possible runtime environment for our application.
#[derive(PartialEq, Clone, Debug, Deserialize)]
pub enum Environment {
    Local,
    Production,
    Test,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
            Environment::Test => "test",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> std::result::Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            "test" => Ok(Self::Test),
            other => Err(format!(
                "{} is not a supported environment. Use either `local` or `production`.",
                other
            )),
        }
    }
}
