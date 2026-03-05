use crate::config::log_level::LogLevel;
use crate::config::store_engine::StoreEngine;
use secrecy::SecretString;
use std::net::{Ipv4Addr, Ipv6Addr};

pub const AUTH_SERVICE_HOST_IPV4: &str = "AUTH_SERVICE_HOST_IPV4";
pub const AUTH_SERVICE_HOST_IPV6: &str = "AUTH_SERVICE_HOST_IPV6";
pub const AUTH_SERVICE_PORT: &str = "AUTH_SERVICE_PORT";
pub const AUTH_SERVICE_LOG: &str = "AUTH_SERVICE_LOG";
pub const AUTH_SERVICE_DB_HOSTNAME: &str = "AUTH_SERVICE_DB_HOSTNAME";
pub const AUTH_SERVICE_DB_PORT: &str = "AUTH_SERVICE_DB_PORT";
pub const AUTH_SERVICE_DB_DATABASE: &str = "AUTH_SERVICE_DB_NAME";
pub const AUTH_SERVICE_DB_USERNAME: &str = "AUTH_SERVICE_DB_USERNAME";
pub const AUTH_SERVICE_DB_PASSWORD: &str = "AUTH_SERVICE_DB_PASSWORD";
pub const AUTH_SERVICE_DB_POOL_MIN_SIZE: &str = "AUTH_SERVICE_DB_POOL_MIN_SIZE";
pub const AUTH_SERVICE_DB_POOL_MAX_SIZE: &str = "AUTH_SERVICE_DB_POOL_MAX_SIZE";
pub const AUTH_SERVICE_JWT_TTL_SECONDS: &str = "AUTH_SERVICE_JWT_TTL_SECONDS";
pub const AUTH_SERVICE_JWT_SECRET: &str = "AUTH_SERVICE_JWT_SECRET";
pub const AUTH_SERVICE_STORE_ENGINE: &str = "AUTH_SERVICE_STORE_ENGINE";
pub const APP_SERVICE_PORT: &str = "APP_SERVICE_PORT";

pub const AUTH_SERVICE_HOST_IPV4_DEFAULT: Option<Ipv4Addr> = Some(Ipv4Addr::LOCALHOST);
pub const AUTH_SERVICE_HOST_IPV6_DEFAULT: Option<Ipv6Addr> = None;
pub const AUTH_SERVICE_PORT_DEFAULT: u16 = 3000;
pub const AUTH_SERVICE_LOG_DEFAULT: LogLevel = LogLevel::Info;
pub const AUTH_SERVICE_DB_HOSTNAME_DEFAULT: &str = "localhost";
pub const AUTH_SERVICE_DB_PORT_DEFAULT: u16 = 5432;
pub const AUTH_SERVICE_DB_DATABASE_DEFAULT: &str = "letsgetrusty";
pub const AUTH_SERVICE_DB_USERNAME_DEFAULT: &str = "administrator";
pub const AUTH_SERVICE_DB_PASSWORD_DEFAULT: Option<SecretString> = None;
pub const AUTH_SERVICE_DB_POOL_MIN_SIZE_DEFAULT: u32 = 1;
pub const AUTH_SERVICE_DB_POOL_MAX_SIZE_DEFAULT: u32 = 10;
pub const AUTH_SERVICE_JWT_TTL_SECONDS_DEFAULT: i64 = 900; // 15 minutes
pub const AUTH_SERVICE_JWT_SECRET_DEFAULT: Option<SecretString> = None;
pub const AUTH_SERVICE_STORE_ENGINE_DEFAULT: StoreEngine = StoreEngine::Database;
pub const APP_SERVICE_PORT_DEFAULT: u16 = 8000;
