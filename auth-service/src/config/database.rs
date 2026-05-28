use crate::config;
use crate::domain::Secret;
use clap::ValueEnum;
use percent_encoding::AsciiSet;
use percent_encoding::CONTROLS;

// Per RFC 3986 §3.2.1, the userinfo subcomponent permits `unreserved`, `sub-delims`, `:`, and
// percent-encoded octets. `:` separates user from password, so we encode it too.
const USERINFO_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'<')
    .add(b'>')
    .add(b'`')
    .add(b'?')
    .add(b'{')
    .add(b'}')
    .add(b'/')
    .add(b':')
    .add(b';')
    .add(b'=')
    .add(b'@')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'^')
    .add(b'|');

pub mod var {
    pub const ENGINE: &str = "AUTH_SERVICE_DB_ENGINE";
    pub const HOST: &str = "AUTH_SERVICE_DB_HOST";
    pub const NAME: &str = "AUTH_SERVICE_DB_NAME";
    pub const PASSWORD: &str = "AUTH_SERVICE_DB_PASSWORD";
    pub const POOL_MAX: &str = "AUTH_SERVICE_DB_POOL_MAX";
    pub const POOL_MIN: &str = "AUTH_SERVICE_DB_POOL_MIN";
    pub const PORT: &str = "AUTH_SERVICE_DB_PORT";
    pub const USER: &str = "AUTH_SERVICE_DB_USER";
}

pub mod default {
    use super::DatabaseEngine;
    pub const ENGINE: DatabaseEngine = DatabaseEngine::Memory;
    pub const HOST: &str = "127.0.0.1";
    pub const NAME: &str = "letsgetrusty";
    pub const POOL_MAX: u32 = 10;
    pub const POOL_MIN: u32 = 1;
    pub const PORT: u16 = 5432;
    pub const USER: &str = "administrator";
}

#[derive(clap::ValueEnum, Clone, Debug, PartialEq)]
#[value(rename_all = "kebab-case")]
pub enum DatabaseEngine {
    Memory,
    Postgres,
}

impl std::fmt::Display for DatabaseEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_possible_value().unwrap().get_name())
    }
}

#[derive(clap::Args, Debug)]
pub struct DatabaseConfig {
    #[arg(
        long,
        env = var::ENGINE,
        default_value_t = default::ENGINE,
        help = "Engine to be used as persistent store.",
        value_parser = clap::value_parser!(DatabaseEngine),
    )]
    pub db_engine: DatabaseEngine,

    #[arg(
        long,
        env = var::HOST,
        default_value_t = String::from(default::HOST),
        help = "Hostname of the database server.",
    )]
    pub db_host: String,

    #[arg(
        long,
        env = var::NAME,
        default_value_t = String::from(default::NAME),
        help = "Name of the database instance.",
    )]
    pub db_name: String,

    #[arg(skip)]
    pub db_password: Option<Secret>,

    #[arg(
        long,
        env = var::POOL_MAX,
        default_value_t = default::POOL_MAX,
        help = "Maximum number of connections to the database.",
        value_parser = clap::value_parser!(u32).range(1..=100),
    )]
    pub db_pool_max: u32,

    #[arg(
        long,
        env = var::POOL_MIN,
        default_value_t = default::POOL_MIN,
        help = "Minimum number of connections to the database.",
        value_parser = clap::value_parser!(u32).range(1..=100),
    )]
    pub db_pool_min: u32,

    #[arg(
        long,
        env = var::PORT,
        default_value_t = default::PORT,
        help = "Port of the database server.",
        value_parser = clap::value_parser!(u16).range(1024..),
    )]
    pub db_port: u16,

    #[arg(
        long,
        env = var::USER,
        default_value_t = String::from(default::USER),
        help = "Username to access the database.",
    )]
    pub db_user: String,
}

impl DatabaseConfig {
    #[must_use]
    pub fn from_environment() -> Self {
        let db_engine = std::env::var(var::ENGINE)
            .ok()
            .and_then(|s| DatabaseEngine::from_str(&s, true).ok())
            .unwrap_or_else(|| {
                tracing::warn!("using default value: {}={}", var::ENGINE, default::ENGINE,);
                default::ENGINE
            });

        let db_host = std::env::var(var::HOST).unwrap_or_else(|_| {
            tracing::warn!("using default value: {}={}", var::HOST, default::HOST,);
            default::HOST.to_owned()
        });

        let db_name = std::env::var(var::NAME).unwrap_or_else(|_| {
            tracing::warn!("using default value: {}={}", var::NAME, default::NAME,);
            default::NAME.to_owned()
        });

        let db_pool_max = std::env::var(var::POOL_MAX)
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .filter(|&size| (1..=100).contains(&size))
            .unwrap_or_else(|| {
                tracing::warn!("using default value: {}={}", var::POOL_MAX, default::POOL_MAX,);
                default::POOL_MAX
            });

        let db_pool_min = std::env::var(var::POOL_MIN)
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .filter(|&size| (1..=100).contains(&size))
            .unwrap_or_else(|| {
                tracing::warn!("using default value: {}={}", var::POOL_MIN, default::POOL_MIN,);
                default::POOL_MIN
            });

        let db_port = std::env::var(var::PORT)
            .ok()
            .and_then(|s| s.parse::<u16>().ok())
            .filter(|&p| p >= 1024)
            .unwrap_or_else(|| {
                tracing::warn!("using default value: {}={}", var::PORT, default::PORT,);
                default::PORT
            });

        let db_user = std::env::var(var::USER).unwrap_or_else(|_| {
            tracing::warn!("using default value: {}={}", var::USER, default::USER,);
            default::USER.to_owned()
        });

        Self {
            db_engine,
            db_host,
            db_name,
            db_password: None,
            db_pool_max,
            db_pool_min,
            db_port,
            db_user,
        }
    }

    pub fn load_mandatory_arguments(&mut self) {
        if self.db_engine == DatabaseEngine::Postgres {
            let db_password = config::secret_from_environment(var::PASSWORD);
            self.db_password = db_password;
        }
    }

    #[must_use]
    pub fn db_url(&self, store_name: Option<&str>) -> Secret {
        let db_name = store_name.unwrap_or(&self.db_name);
        let password = self.db_password.as_ref().unwrap_or_else(|| {
            tracing::error!("{} is not set in the environment", var::PASSWORD);
            panic!("{} is not set in the environment", var::PASSWORD)
        });
        // Percent-encode the user/password so URL-significant bytes (`@`, `:`, `/`, `?`, etc.)
        // can't malform the connection string.
        let user = percent_encoding::utf8_percent_encode(&self.db_user, USERINFO_ENCODE_SET);
        let password = percent_encoding::utf8_percent_encode(password.expose(), USERINFO_ENCODE_SET);
        format!(
            "postgresql://{}:{}@{}:{}/{}?options=-c%20search_path%3Dauth,public",
            user, password, self.db_host, self.db_port, db_name,
        )
        .into()
    }
}

impl std::fmt::Display for DatabaseConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DatabaseConfig")
            .field("db_engine", &self.db_engine)
            .field("db_host", &self.db_host)
            .field("db_name", &self.db_name)
            .field("db_password", &self.db_password)
            .field("db_pool_max", &self.db_pool_max)
            .field("db_pool_min", &self.db_pool_min)
            .field("db_port", &self.db_port)
            .field("db_user", &self.db_user)
            .finish()
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            db_engine: default::ENGINE,
            db_host: String::from(default::HOST),
            db_name: String::from(default::NAME),
            db_password: None,
            db_pool_max: default::POOL_MAX,
            db_pool_min: default::POOL_MIN,
            db_port: default::PORT,
            db_user: String::from(default::USER),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config_with(user: &str, password: &str) -> DatabaseConfig {
        DatabaseConfig {
            db_engine: DatabaseEngine::Postgres,
            db_host: "127.0.0.1".to_owned(),
            db_name: "letsgetrusty".to_owned(),
            db_password: Some(password.into()),
            db_pool_max: 10,
            db_pool_min: 1,
            db_port: 5432,
            db_user: user.to_owned(),
        }
    }

    #[test]
    fn plain_password_is_passed_through() {
        let url = config_with("alice", "simplepass").db_url(None);
        assert!(
            url.expose().contains("alice:simplepass@127.0.0.1:5432"),
            "got: {}",
            url.expose()
        );
    }

    #[test]
    fn password_with_at_sign_is_encoded() {
        let url = config_with("alice", "p@ss/word").db_url(None);
        // '@', '/', '?', etc. must be percent-encoded so the URL parser doesn't split on them.
        assert!(!url.expose().contains("p@ss/word"), "raw '@' leaked: {}", url.expose());
        assert!(url.expose().contains("p%40ss%2Fword"), "got: {}", url.expose());
    }

    #[test]
    fn password_with_colon_and_question_mark_is_encoded() {
        let url = config_with("alice", "a:b?c").db_url(None);
        assert!(url.expose().contains("a%3Ab%3Fc"), "got: {}", url.expose());
    }

    #[test]
    fn user_special_characters_are_encoded() {
        let url = config_with("user@host", "pwd").db_url(None);
        assert!(url.expose().contains("user%40host:pwd"), "got: {}", url.expose());
    }

    #[test]
    fn store_name_override_is_used_in_path() {
        let url = config_with("alice", "pwd").db_url(Some("test_db"));
        assert!(url.expose().contains("/test_db?"), "got: {}", url.expose());
    }
}
