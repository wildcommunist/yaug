use std::time::Duration;
use secrecy::{ExposeSecret, Secret};
use sqlx::{ConnectOptions, PgPool};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};
use serde_aux::field_attributes::deserialize_number_from_string;
use tera::Tera;
use crate::domain::UserEmail;
use crate::email_client::EmailClient;

// region Enums & Implementations
pub enum Environment {
    Local,
    Testing,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Testing => "testing",
            Environment::Production => "production"
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "local" => Ok(Environment::Local),
            "testing" => Ok(Environment::Testing),
            "production" => Ok(Environment::Production),
            other => {
                Err(
                    format!(
                        "{} is not a valid environment configuration.\n Use 'local', 'testing' or 'production'",
                        other
                    )
                )
            }
        }
    }
}
//endregion

//region Structs & Implementations
#[derive(serde::Deserialize, Clone)]
pub struct Settings {
    pub app: ApplicationSettings,
    pub db: DatabaseSettings,
    pub email: EmailSettings,
}

impl Settings {
    pub fn current_environment() -> Environment {
        std::env::var("APP_ENVIRONMENT")
            .unwrap_or_else(|_| "local".into())
            .try_into()
            .expect("Failed to parse `APP_ENVIRONMENT`.")
    }

    pub fn get_template_engine(&self) -> Tera {
        let template_dir = &self.app.template_base_dir;
        Tera::new(template_dir).unwrap()
    }
}

#[derive(serde::Deserialize, Clone)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub redis_uri: Secret<String>,
    pub cookie_secret: Secret<String>,
    pub template_base_dir: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct DatabaseSettings {
    pub host: String,
    pub username: String,
    pub password: Secret<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub database: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub pool_timeout_seconds: u64,
    pub require_ssl: bool,
}

impl DatabaseSettings {
    pub fn get_conn_str_with_db(&self) -> PgConnectOptions {
        let mut options = self.get_conn_str_without_db().database(&self.database);
        options.log_statements(tracing::log::LevelFilter::Trace);
        options
    }

    pub fn get_conn_str_without_db(&self) -> PgConnectOptions {
        let ssl = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };

        PgConnectOptions::new()
            .host(&self.host)
            .port(self.port)
            .username(&self.username)
            .password(&self.password.expose_secret())
            .ssl_mode(ssl)
    }

    pub fn get_connection_pool(&self) -> PgPool {
        PgPoolOptions::new()
            .acquire_timeout(
                std::time::Duration::from_secs(self.pool_timeout_seconds)
            )
            .connect_lazy_with(self.get_conn_str_with_db())
    }
}

#[derive(serde::Deserialize, Clone)]
pub struct EmailSettings {
    pub base_url: String,
    pub sender_email: String,
    pub auth_token: Secret<String>,
    pub timeout_milliseconds: u64,
}

impl EmailSettings {
    pub fn sender(&self) -> Result<UserEmail, String> {
        UserEmail::parse(self.sender_email.clone())
    }

    pub fn timeout(&self) -> Duration {
        Duration::from_millis(self.timeout_milliseconds)
    }

    pub fn client(self) -> EmailClient {
        let sender_email = self.sender().expect("Invalid sender email address");
        let timeout = self.timeout();
        EmailClient::new(
            self.base_url,
            sender_email,
            self.auth_token,
            timeout,
        )
    }
}

//endregion

//region functions
pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine current directory.");
    let configuration_directory = base_path.join("configuration");
    let environment = Settings::current_environment();
    let environment_filename = format!("{}.toml", environment.as_str());

    /*
    Order of loading:
    1) load the config.toml file
    2) load <environment>.toml file [optional]
    3) override config vars from environment in format APP_SETTINGNAME or APP_SECTION__SECTION_VALUE
     */

    let settings = config::Config::builder()
        .add_source(
            config::File::from(configuration_directory.join("config.toml"))
        )
        .add_source(
            config::File::from(configuration_directory.join(&environment_filename)).required(false)
        )
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__")
        )
        .build()?;

    settings.try_deserialize::<Settings>()
}
//endregion
