pub mod consts;

mod log_level;
pub use log_level::LogLevel;

mod store_engine;
pub use store_engine::StoreEngine;

use clap::ArgGroup;
use clap::Parser;
use clap::ValueEnum;
use dotenvy::dotenv_override;
use fmt::{Display, Formatter};
use secrecy::{ExposeSecret, SecretString};
use std::env;
use std::fmt;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::sync::Arc;
use tracing::{error, info, instrument, warn};

#[allow(unused_imports)]
use tracing::Level;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(group(
    ArgGroup::new("ip_address_selection")
    .args(["ipv4","ipv6"])
    .required(false)
    .multiple(false)
))]
pub struct Config {
    #[arg(
        long,
        env = consts::AUTH_SERVICE_HOST_IPV4,
        help = "IPv4 address for the auth service to listen on.",
    )]
    pub ipv4: Option<Ipv4Addr>,

    #[arg(
        long,
        env = consts::AUTH_SERVICE_HOST_IPV6,
        help = "IPv6 address for the auth service to listen on.",
    )]
    pub ipv6: Option<Ipv6Addr>,

    #[arg(
        long,
        env = consts::AUTH_SERVICE_PORT,
        default_value_t = consts::AUTH_SERVICE_PORT_DEFAULT,
        help = "Port for the auth service to listen on.",
        value_parser = clap::value_parser!(u16).range(1024..),
    )]
    pub port: u16,

    #[arg(
        long,
        env = consts::AUTH_SERVICE_LOG,
        default_value_t = consts::AUTH_SERVICE_LOG_DEFAULT,
        help = "Log level for the auth service.",
    )]
    pub log: LogLevel,

    #[arg(
        long,
        env = consts::AUTH_SERVICE_CACHE_HOSTNAME,
        default_value_t = String::from(consts::AUTH_SERVICE_CACHE_HOSTNAME_DEFAULT),
        help = "Hostname of the cache server.",
    )]
    pub cache_hostname: String,

    #[arg(
        long,
        env = consts::AUTH_SERVICE_CACHE_PORT,
        default_value_t = consts::AUTH_SERVICE_CACHE_PORT_DEFAULT,
        help = "Port of the cache server.",
        value_parser = clap::value_parser!(u16).range(1024..),
    )]
    pub cache_port: u16,

    #[arg(
        long,
        env = consts::AUTH_SERVICE_DB_HOSTNAME,
        default_value_t = String::from(consts::AUTH_SERVICE_DB_HOSTNAME_DEFAULT),
        help = "Hostname of the database server.",
    )]
    pub db_hostname: String,

    #[arg(
        long,
        env = consts::AUTH_SERVICE_DB_PORT,
        default_value_t = consts::AUTH_SERVICE_DB_PORT_DEFAULT,
        help = "Port of the database server.",
        value_parser = clap::value_parser!(u16).range(1024..),
    )]
    pub db_port: u16,

    #[arg(
        long,
        env = consts::AUTH_SERVICE_DB_DATABASE,
        default_value_t = String::from(consts::AUTH_SERVICE_DB_DATABASE_DEFAULT),
        help = "Name of the database server.",
    )]
    pub db_database: String,

    #[arg(
        long,
        env = consts::AUTH_SERVICE_DB_USERNAME,
        default_value_t = String::from(consts::AUTH_SERVICE_DB_USERNAME_DEFAULT),
        help = "Username to access the database server.",
    )]
    pub db_username: String,

    #[arg(skip)]
    pub db_password: Option<SecretString>,

    #[arg(
        long,
        env = consts::AUTH_SERVICE_DB_POOL_MIN_SIZE,
        default_value_t = consts::AUTH_SERVICE_DB_POOL_MIN_SIZE_DEFAULT,
        help = "Minimum number of connections in the database pool.",
        value_parser = clap::value_parser!(u32).range(1..10),
    )]
    pub db_pool_min_size: u32,

    #[arg(
        long,
        env = consts::AUTH_SERVICE_DB_POOL_MAX_SIZE,
        default_value_t = consts::AUTH_SERVICE_DB_POOL_MAX_SIZE_DEFAULT,
        help = "Maximum number of connections in the database pool.",
        value_parser = clap::value_parser!(u32).range(1..100),
    )]
    pub db_pool_max_size: u32,

    #[arg(
        long,
        env = consts::AUTH_SERVICE_JWT_TTL_SECONDS,
        default_value_t = consts::AUTH_SERVICE_JWT_TTL_SECONDS_DEFAULT,
        help = "Time-to-live (TTL) for JWT tokens in seconds.",
        value_parser = clap::value_parser!(i64).range(300..3600),
    )]
    pub jwt_ttl_seconds: i64,

    #[arg(skip)]
    pub jwt_secret: Option<SecretString>,

    #[arg(
        long,
        env = consts::AUTH_SERVICE_STORE_ENGINE,
        default_value_t = consts::AUTH_SERVICE_STORE_ENGINE_DEFAULT,
        help = "Engine to be used by the data stores.",
    )]
    pub store_engine: StoreEngine,

    #[arg(
        long,
        env = consts::APP_SERVICE_PORT,
        default_value_t = consts::APP_SERVICE_PORT_DEFAULT,
        help = "Port where the app service listens on.",
        value_parser = clap::value_parser!(u16).range(1024..),
    )]
    pub app_service_port: u16,
}

impl Display for Config {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("app_service_port", &self.app_service_port)
            .field("cache_hostname", &self.cache_hostname)
            .field("cache_port", &self.cache_port)
            .field("db_database", &self.db_database)
            .field("db_hostname", &self.db_hostname)
            .field("db_password", &self.db_password)
            .field("db_pool_max_size", &self.db_pool_max_size)
            .field("db_pool_min_size", &self.db_pool_min_size)
            .field("db_port", &self.db_port)
            .field("db_username", &self.db_username)
            .field("ipv4", &self.ipv4)
            .field("ipv6", &self.ipv6)
            .field("jwt_secret", &self.jwt_secret)
            .field("jwt_ttl_seconds", &self.jwt_ttl_seconds)
            .field("log", &self.log)
            .field("port", &self.port)
            .field("store_engine", &self.store_engine)
            .finish()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            app_service_port: consts::APP_SERVICE_PORT_DEFAULT,
            cache_hostname: String::from(consts::AUTH_SERVICE_CACHE_HOSTNAME_DEFAULT),
            cache_port: consts::AUTH_SERVICE_CACHE_PORT_DEFAULT,
            db_database: String::from(consts::AUTH_SERVICE_DB_DATABASE_DEFAULT),
            db_hostname: String::from(consts::AUTH_SERVICE_DB_HOSTNAME_DEFAULT),
            db_password: consts::AUTH_SERVICE_DB_PASSWORD_DEFAULT,
            db_pool_max_size: consts::AUTH_SERVICE_DB_POOL_MAX_SIZE_DEFAULT,
            db_pool_min_size: consts::AUTH_SERVICE_DB_POOL_MIN_SIZE_DEFAULT,
            db_port: consts::AUTH_SERVICE_DB_PORT_DEFAULT,
            db_username: String::from(consts::AUTH_SERVICE_DB_USERNAME_DEFAULT),
            ipv4: consts::AUTH_SERVICE_HOST_IPV4_DEFAULT,
            ipv6: consts::AUTH_SERVICE_HOST_IPV6_DEFAULT,
            jwt_secret: consts::AUTH_SERVICE_JWT_SECRET_DEFAULT,
            jwt_ttl_seconds: consts::AUTH_SERVICE_JWT_TTL_SECONDS_DEFAULT,
            log: consts::AUTH_SERVICE_LOG_DEFAULT,
            port: consts::AUTH_SERVICE_PORT_DEFAULT,
            store_engine: consts::AUTH_SERVICE_STORE_ENGINE_DEFAULT,
        }
    }
}

impl Config {
    #[instrument(level = Level::TRACE)]
    pub fn init_from_env() -> Self {
        let dotenv = dotenv_override().ok();
        if let Some(dotenv) = dotenv {
            info!("Initialized: {}", dotenv.display());
        }

        let mut ipv4 = env::var(consts::AUTH_SERVICE_HOST_IPV4)
            .ok()
            .and_then(|s| s.parse::<Ipv4Addr>().ok());

        let mut ipv6 = env::var(consts::AUTH_SERVICE_HOST_IPV6)
            .ok()
            .and_then(|s| s.parse::<Ipv6Addr>().ok());

        if ipv4.is_none() && ipv6.is_none() {
            ipv4 = consts::AUTH_SERVICE_HOST_IPV4_DEFAULT;
            ipv6 = consts::AUTH_SERVICE_HOST_IPV6_DEFAULT;
            warn!(
                "both {} and {} are not set",
                consts::AUTH_SERVICE_HOST_IPV4,
                consts::AUTH_SERVICE_HOST_IPV6,
            );
            warn!(
                "using IPv4 default value: {}={:?}",
                consts::AUTH_SERVICE_HOST_IPV4,
                consts::AUTH_SERVICE_HOST_IPV4_DEFAULT,
            );
        }

        if ipv4.is_some() && ipv6.is_some() {
            ipv6 = consts::AUTH_SERVICE_HOST_IPV6_DEFAULT;
            warn!(
                "both {} and {} are set",
                consts::AUTH_SERVICE_HOST_IPV4,
                consts::AUTH_SERVICE_HOST_IPV6,
            );
            warn!("invalidating IPv6");
        }

        let port = env::var(consts::AUTH_SERVICE_PORT)
            .ok()
            .and_then(|s| s.parse::<u16>().ok())
            .filter(|&p| p >= 1024)
            .unwrap_or_else(|| {
                warn!(
                    "using default value: {}={}",
                    consts::AUTH_SERVICE_PORT,
                    consts::AUTH_SERVICE_PORT_DEFAULT,
                );
                consts::AUTH_SERVICE_PORT_DEFAULT
            });

        let log = env::var(consts::AUTH_SERVICE_LOG)
            .ok()
            .and_then(|s| LogLevel::from_str(&s, true).ok())
            .unwrap_or_else(|| {
                warn!(
                    "using default value: {}={}",
                    consts::AUTH_SERVICE_LOG,
                    consts::AUTH_SERVICE_LOG_DEFAULT,
                );
                consts::AUTH_SERVICE_LOG_DEFAULT
            });

        let cache_hostname = env::var(consts::AUTH_SERVICE_CACHE_HOSTNAME).unwrap_or_else(|_| {
            warn!(
                "using default value: {}={}",
                consts::AUTH_SERVICE_CACHE_HOSTNAME,
                consts::AUTH_SERVICE_CACHE_HOSTNAME_DEFAULT,
            );
            consts::AUTH_SERVICE_CACHE_HOSTNAME_DEFAULT.to_string()
        });

        let cache_port = env::var(consts::AUTH_SERVICE_CACHE_PORT)
            .ok()
            .and_then(|s| s.parse::<u16>().ok())
            .filter(|&p| p >= 1024)
            .unwrap_or_else(|| {
                warn!(
                    "using default value: {}={}",
                    consts::AUTH_SERVICE_CACHE_PORT,
                    consts::AUTH_SERVICE_CACHE_PORT_DEFAULT,
                );
                consts::AUTH_SERVICE_CACHE_PORT_DEFAULT
            });

        let db_hostname = env::var(consts::AUTH_SERVICE_DB_HOSTNAME).unwrap_or_else(|_| {
            warn!(
                "using default value: {}={}",
                consts::AUTH_SERVICE_DB_HOSTNAME,
                consts::AUTH_SERVICE_DB_HOSTNAME_DEFAULT,
            );
            consts::AUTH_SERVICE_DB_HOSTNAME_DEFAULT.to_string()
        });

        let db_port = env::var(consts::AUTH_SERVICE_DB_PORT)
            .ok()
            .and_then(|s| s.parse::<u16>().ok())
            .filter(|&p| p >= 1024)
            .unwrap_or_else(|| {
                warn!(
                    "using default value: {}={}",
                    consts::AUTH_SERVICE_DB_PORT,
                    consts::AUTH_SERVICE_DB_PORT_DEFAULT,
                );
                consts::AUTH_SERVICE_DB_PORT_DEFAULT
            });

        let db_name = env::var(consts::AUTH_SERVICE_DB_DATABASE).unwrap_or_else(|_| {
            warn!(
                "using default value: {}={}",
                consts::AUTH_SERVICE_DB_DATABASE,
                consts::AUTH_SERVICE_DB_DATABASE_DEFAULT,
            );
            consts::AUTH_SERVICE_DB_DATABASE_DEFAULT.to_string()
        });

        let db_username = env::var(consts::AUTH_SERVICE_DB_USERNAME).unwrap_or_else(|_| {
            warn!(
                "using default value: {}={}",
                consts::AUTH_SERVICE_DB_USERNAME,
                consts::AUTH_SERVICE_DB_USERNAME_DEFAULT,
            );
            consts::AUTH_SERVICE_DB_USERNAME_DEFAULT.to_string()
        });

        let db_pool_min_size = env::var(consts::AUTH_SERVICE_DB_POOL_MIN_SIZE)
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .filter(|&size| (1..10).contains(&size))
            .unwrap_or_else(|| {
                warn!(
                    "using default value: {}={}",
                    consts::AUTH_SERVICE_DB_POOL_MIN_SIZE,
                    consts::AUTH_SERVICE_DB_POOL_MIN_SIZE_DEFAULT,
                );
                consts::AUTH_SERVICE_DB_POOL_MIN_SIZE_DEFAULT
            });

        let db_pool_max_size = env::var(consts::AUTH_SERVICE_DB_POOL_MAX_SIZE)
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .filter(|&size| (1..100).contains(&size))
            .unwrap_or_else(|| {
                warn!(
                    "using default value: {}={}",
                    consts::AUTH_SERVICE_DB_POOL_MAX_SIZE,
                    consts::AUTH_SERVICE_DB_POOL_MAX_SIZE_DEFAULT,
                );
                consts::AUTH_SERVICE_DB_POOL_MAX_SIZE_DEFAULT
            });

        let jwt_ttl_seconds = env::var(consts::AUTH_SERVICE_JWT_TTL_SECONDS)
            .ok()
            .and_then(|s| s.parse::<i64>().ok())
            .filter(|&ttl| (300..3600).contains(&ttl))
            .unwrap_or_else(|| {
                warn!(
                    "using default value: {}={}",
                    consts::AUTH_SERVICE_JWT_TTL_SECONDS,
                    consts::AUTH_SERVICE_JWT_TTL_SECONDS_DEFAULT,
                );
                consts::AUTH_SERVICE_JWT_TTL_SECONDS_DEFAULT
            });

        let store_engine = env::var(consts::AUTH_SERVICE_STORE_ENGINE)
            .ok()
            .and_then(|s| StoreEngine::from_str(&s, true).ok())
            .unwrap_or_else(|| {
                warn!(
                    "using default value: {}={}",
                    consts::AUTH_SERVICE_STORE_ENGINE,
                    consts::AUTH_SERVICE_STORE_ENGINE_DEFAULT,
                );
                consts::AUTH_SERVICE_STORE_ENGINE_DEFAULT
            });

        let app_service_port = env::var(consts::APP_SERVICE_PORT)
            .ok()
            .and_then(|s| s.parse::<u16>().ok())
            .filter(|&p| p >= 1024)
            .unwrap_or_else(|| {
                warn!(
                    "using default value: {}={}",
                    consts::APP_SERVICE_PORT,
                    consts::APP_SERVICE_PORT_DEFAULT,
                );
                consts::APP_SERVICE_PORT_DEFAULT
            });

        let db_password = secret_from_environment(consts::AUTH_SERVICE_DB_PASSWORD);
        let jwt_secret = secret_from_environment(consts::AUTH_SERVICE_JWT_SECRET);

        Self {
            app_service_port,
            cache_hostname,
            cache_port,
            db_database: db_name,
            db_hostname,
            db_password,
            db_pool_max_size,
            db_pool_min_size,
            db_port,
            db_username,
            ipv4,
            ipv6,
            jwt_secret,
            jwt_ttl_seconds,
            log,
            port,
            store_engine,
        }
    }

    #[instrument(level = Level::TRACE)]
    pub fn init_from_env_and_cli() -> Self {
        let dotenv = dotenv_override().ok();
        if let Some(dotenv) = dotenv {
            info!("Initialized: {}", dotenv.display());
        }
        let mut config = Self::parse();
        let db_password = secret_from_environment(consts::AUTH_SERVICE_DB_PASSWORD);
        let jwt_secret = secret_from_environment(consts::AUTH_SERVICE_JWT_SECRET);
        assert!(
            !(db_password.is_none() || jwt_secret.is_none()),
            "{} and {} must be set in the environment",
            consts::AUTH_SERVICE_DB_PASSWORD,
            consts::AUTH_SERVICE_JWT_SECRET
        );
        config.db_password = db_password;
        config.jwt_secret = jwt_secret;
        config
    }

    #[must_use]
    pub fn database_url(&self, db_database: Option<&str>) -> String {
        let db_database = db_database.unwrap_or(&self.db_database);
        format!(
            "postgresql://{}:{}@{}:{}/{}?options=-c%20search_path%3Dauth,public",
            self.db_username,
            self.db_password.as_ref().unwrap().expose_secret(),
            self.db_hostname,
            self.db_port,
            db_database
        )
    }

    #[must_use]
    pub fn cache_url(&self) -> String {
        format!("redis://{}:{}", self.cache_hostname, self.cache_port)
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

#[instrument(level = Level::TRACE)]
pub fn secret_from_environment(environment_variable: &str) -> Option<SecretString> {
    let secret = match env::var(environment_variable) {
        Ok(string) => string,
        Err(error) => {
            error!("{}: {}", environment_variable, error.to_string());
            return None;
        }
    };

    if secret.trim().is_empty() {
        error!("{}: {}", environment_variable, "is empty");
        return None;
    }

    Some(SecretString::from(secret))
}
