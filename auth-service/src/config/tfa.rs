pub mod var {
    pub const TTL: &str = "AUTH_SERVICE_TFA_TTL";
}

pub mod default {
    pub const TTL: u32 = 300; // 5 minutes
}

#[derive(clap::Args, Debug)]
pub struct TfaConfig {
    #[arg(
        long,
        env = var::TTL,
        default_value_t = default::TTL,
        help = "TTL for TFA codes in seconds.",
        value_parser = clap::value_parser!(u32).range(60..=900),
    )]
    pub tfa_ttl: u32,
}

impl TfaConfig {
    #[must_use]
    pub fn from_environment() -> Self {
        let tfa_ttl = std::env::var(var::TTL)
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .filter(|&ttl| (60..=900).contains(&ttl))
            .unwrap_or_else(|| {
                tracing::warn!("using default value: {}={}", var::TTL, default::TTL,);
                default::TTL
            });
        Self { tfa_ttl }
    }
}

impl std::fmt::Display for TfaConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TfaConfig")
            .field("tfa_ttl_secs", &self.tfa_ttl)
            .finish()
    }
}

impl Default for TfaConfig {
    fn default() -> Self {
        Self { tfa_ttl: default::TTL }
    }
}
