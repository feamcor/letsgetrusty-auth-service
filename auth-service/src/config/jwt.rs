use crate::config;
use crate::domain::Secret;

pub mod var {
    pub const SECRET: &str = "AUTH_SERVICE_JWT_SECRET";
    pub const TTL: &str = "AUTH_SERVICE_JWT_TTL";
}

pub mod default {
    pub const TTL: u32 = 900; // 15 minutes
}

/// Minimum acceptable JWT signing-key length. Per RFC 7518 §3.2 the HS256 key SHOULD be at
/// least 32 bytes (256 bits); shorter keys are trivially brute-forceable for HMAC signatures.
pub const MIN_JWT_SECRET_BYTES: usize = 32;

/// Single source of truth for the accepted jwt_ttl range. Typed as i64 so clap's value_parser
/// (which constrains via RangeBounds<i64>) and the env-var fallback can share the same literal.
pub const JWT_TTL_RANGE: std::ops::RangeInclusive<i64> = 300..=3600;

#[derive(clap::Args, Debug)]
pub struct JwtConfig {
    #[arg(skip)]
    pub jwt_secret: Option<Secret>,

    #[arg(
        long,
        env = var::TTL,
        default_value_t = default::TTL,
        help = "TTL for JWTs in seconds.",
        value_parser = clap::value_parser!(u32).range(JWT_TTL_RANGE),
    )]
    pub jwt_ttl: u32,
}

impl JwtConfig {
    #[must_use]
    pub fn from_environment() -> Self {
        let jwt_ttl = std::env::var(var::TTL)
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .filter(|&ttl| JWT_TTL_RANGE.contains(&i64::from(ttl)))
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
        if let Some(secret) = jwt_secret.as_ref()
            && secret.expose().len() < MIN_JWT_SECRET_BYTES
        {
            tracing::error!(
                "{} must be at least {} bytes (got {})",
                var::SECRET,
                MIN_JWT_SECRET_BYTES,
                secret.expose().len()
            );
            panic!(
                "{} must be at least {} bytes",
                var::SECRET,
                MIN_JWT_SECRET_BYTES
            );
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn min_secret_length_matches_rfc_7518() {
        // RFC 7518 §3.2 recommends >=256 bits = 32 bytes for HS256.
        assert!(MIN_JWT_SECRET_BYTES >= 32);
    }

    #[test]
    #[should_panic(expected = "must be at least 32 bytes")]
    fn load_panics_on_too_short_secret() {
        // SAFETY: env mutation is racy in parallel tests; we use a unique var name
        // and restore it immediately. No other test reads this var.
        const VAR: &str = "AUTH_SERVICE_JWT_SECRET_FOR_SHORT_TEST";
        // SAFETY: see comment above.
        unsafe {
            std::env::set_var(VAR, "tooshort");
        }
        // Manually invoke the validation against the env var we just set.
        let secret = std::env::var(VAR).map(crate::domain::Secret::from).unwrap();
        // SAFETY: see comment above.
        unsafe {
            std::env::remove_var(VAR);
        }
        if secret.expose().len() < MIN_JWT_SECRET_BYTES {
            panic!(
                "{} must be at least {} bytes",
                var::SECRET,
                MIN_JWT_SECRET_BYTES
            );
        }
    }

    #[test]
    fn long_secret_is_accepted() {
        let long: crate::domain::Secret = "a".repeat(MIN_JWT_SECRET_BYTES).into();
        assert!(long.expose().len() >= MIN_JWT_SECRET_BYTES);
    }
}
