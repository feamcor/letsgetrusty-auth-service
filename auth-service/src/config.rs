pub mod cache;
pub mod database;
pub mod email;
pub mod jwt;
pub mod log;
pub mod network;
pub mod tfa;

use crate::domain::Secret;
use clap::Parser;
use dotenvy::dotenv_override;
use std::sync::Arc;

#[derive(Parser, Debug, Default)]
#[command(author, version, about, long_about = None)]
pub struct Config {
    #[command(flatten, next_help_heading = "Log Options")]
    pub log: log::LogConfig,
    #[command(flatten, next_help_heading = "Network Options")]
    pub network: network::NetworkConfig,
    #[command(flatten, next_help_heading = "JWT Options")]
    pub jwt: jwt::JwtConfig,
    #[command(flatten, next_help_heading = "TFA Options")]
    pub tfa: tfa::TfaConfig,
    #[command(flatten, next_help_heading = "Database Options")]
    pub db: database::DatabaseConfig,
    #[command(flatten, next_help_heading = "Cache Options")]
    pub cache: cache::CacheConfig,
    #[command(flatten, next_help_heading = "Email Options")]
    pub email: email::EmailServiceConfig,
}

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("log", &self.log)
            .field("network", &self.network)
            .field("jwt", &self.jwt)
            .field("tfa", &self.tfa)
            .field("db", &self.db)
            .field("cache", &self.cache)
            .field("email", &self.email)
            .finish()
    }
}

impl Config {
    pub fn init_from_env() -> Self {
        let dotenv = dotenv_override().ok();
        if let Some(dotenv) = dotenv {
            tracing::info!("Initialized: {}", dotenv.display());
        }
        let mut config = Self {
            log: log::LogConfig::from_environment(),
            network: network::NetworkConfig::from_environment(),
            jwt: jwt::JwtConfig::from_environment(),
            tfa: tfa::TfaConfig::from_environment(),
            db: database::DatabaseConfig::from_environment(),
            cache: cache::CacheConfig::from_environment(),
            email: email::EmailServiceConfig::from_environment(),
        };
        config.email.load_mandatory_arguments();
        config.jwt.load_mandatory_arguments();
        config.db.load_mandatory_arguments();
        config
    }

    pub fn init_from_env_and_cli() -> Self {
        let dotenv = dotenv_override().ok();
        if let Some(dotenv) = dotenv {
            tracing::info!("Initialized: {}", dotenv.display());
        }
        let mut config = Self::parse();
        config.email.load_mandatory_arguments();
        config.jwt.load_mandatory_arguments();
        config.db.load_mandatory_arguments();
        config
    }
}

#[derive(Clone, Debug)]
pub struct ConfigType {
    inner: Arc<Config>,
}

impl ConfigType {
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self {
            inner: Arc::new(config),
        }
    }

    #[must_use]
    pub fn inner(&self) -> Arc<Config> {
        self.inner.clone()
    }
}

pub fn secret_from_environment(variable: &str) -> Option<Secret> {
    let Ok(secret) = std::env::var(variable) else {
        panic!("secret is not set in the environment: {variable}");
    };
    assert!(
        !secret.trim().is_empty(),
        "secret's value is empty in the environment: {variable}"
    );
    Some(secret.into())
}
