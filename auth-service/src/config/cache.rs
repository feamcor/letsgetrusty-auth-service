use crate::domain::Secret;
use clap::ValueEnum;

pub mod var {
    pub const ENGINE: &str = "AUTH_SERVICE_CACHE_ENGINE";
    pub const HOST: &str = "AUTH_SERVICE_CACHE_HOST";
    pub const PORT: &str = "AUTH_SERVICE_CACHE_PORT";
}

pub mod default {
    use super::CacheEngine;
    pub const ENGINE: CacheEngine = CacheEngine::Memory;
    pub const HOST: &str = "127.0.0.1";
    pub const PORT: u16 = 6379;
}

#[derive(clap::ValueEnum, Clone, Debug, PartialEq)]
#[value(rename_all = "kebab-case")]
pub enum CacheEngine {
    Memory,
    Redis,
}

impl std::fmt::Display for CacheEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_possible_value().unwrap().get_name())
    }
}

#[derive(clap::Args, Debug)]
pub struct CacheConfig {
    #[arg(
        long,
        env = var::ENGINE,
        default_value_t = default::ENGINE,
        help = "Engine to be used as ephemeral store.",
        value_parser = clap::value_parser!(CacheEngine),
    )]
    pub cache_engine: CacheEngine,

    #[arg(
        long,
        env = var::HOST,
        default_value_t = String::from(default::HOST),
        help = "Hostname of the cache server.",
    )]
    pub cache_host: String,

    #[arg(
        long,
        env = var::PORT,
        default_value_t = default::PORT,
        help = "Port of the cache server.",
        value_parser = clap::value_parser!(u16).range(1024..),
    )]
    pub cache_port: u16,
}

impl CacheConfig {
    #[must_use]
    pub fn from_environment() -> Self {
        let cache_engine = std::env::var(var::ENGINE)
            .ok()
            .and_then(|s| CacheEngine::from_str(&s, true).ok())
            .unwrap_or_else(|| {
                tracing::warn!("using default value: {}={}", var::ENGINE, default::ENGINE,);
                default::ENGINE
            });

        let cache_host = std::env::var(var::HOST).unwrap_or_else(|_| {
            tracing::warn!("using default value: {}={}", var::HOST, default::HOST,);
            String::from(default::HOST)
        });

        let cache_port = std::env::var(var::PORT)
            .ok()
            .and_then(|s| s.parse::<u16>().ok())
            .filter(|&p| p >= 1024)
            .unwrap_or_else(|| {
                tracing::warn!("using default value: {}={}", var::PORT, default::PORT,);
                default::PORT
            });

        Self {
            cache_engine,
            cache_host,
            cache_port,
        }
    }

    #[must_use]
    pub fn cache_url(&self) -> Secret {
        format!("redis://{}:{}", self.cache_host, self.cache_port).into()
    }
}

impl std::fmt::Display for CacheConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CacheConfig")
            .field("cache_engine", &self.cache_engine)
            .field("cache_host", &self.cache_host)
            .field("cache_port", &self.cache_port)
            .finish()
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            cache_engine: default::ENGINE,
            cache_host: String::from(default::HOST),
            cache_port: default::PORT,
        }
    }
}
