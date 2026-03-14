use crate::config;
use crate::domain::Secret;

pub mod var {
    pub const SECRET: &str = "AUTH_SERVICE_JWT_SECRET";
    pub const TTL: &str = "AUTH_SERVICE_JWT_TTL";
}

pub mod default {
    pub const TTL: u32 = 900; // 15 minutes
}

#[derive(clap::Args, Debug)]
pub struct JwtConfig {
    #[arg(skip)]
    pub jwt_secret: Option<Secret>,

    #[arg(
        long,
        env = var::TTL,
        default_value_t = default::TTL,
        help = "TTL for JWTs in seconds.",
        value_parser = clap::value_parser!(u32).range(300..3600),
    )]
    pub jwt_ttl: u32,
}

impl JwtConfig {
    #[must_use]
    pub fn from_environment() -> Self {
        let jwt_ttl = std::env::var(var::TTL)
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .filter(|&ttl| (300..3600).contains(&ttl))
            .unwrap_or_else(|| {
                tracing::warn!("using default value: {}={}", var::TTL, default::TTL,);
                default::TTL
            });
        Self {
            jwt_secret: None,
            jwt_ttl,
        }
    }

    pub fn load_mandatory_arguments(&mut self) {
        let jwt_secret = config::secret_from_environment(var::SECRET);
        self.jwt_secret = jwt_secret;
    }
}

impl std::fmt::Display for JwtConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JwtConfig")
            .field("jwt_secret", &self.jwt_secret)
            .field("jwt_ttl", &self.jwt_ttl)
            .finish()
    }
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            jwt_secret: None,
            jwt_ttl: default::TTL,
        }
    }
}
