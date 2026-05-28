pub mod var {
    pub const ALLOWED_ORIGIN: &str = "AUTH_SERVICE_ALLOWED_ORIGIN";
    pub const APP_SERVICE_PORT: &str = "APP_SERVICE_PORT";
    pub const HOST_IPV4: &str = "AUTH_SERVICE_HOST_IPV4";
    pub const HOST_IPV6: &str = "AUTH_SERVICE_HOST_IPV6";
    pub const PORT: &str = "AUTH_SERVICE_PORT";
}

pub mod default {
    pub const APP_SERVICE_PORT: u16 = 8000;
    pub const HOST_IPV4: Option<std::net::Ipv4Addr> = Some(std::net::Ipv4Addr::LOCALHOST);
    pub const PORT: u16 = 3000;
}

#[derive(clap::Args, Debug)]
pub struct NetworkConfig {
    #[arg(
        long,
        env = var::HOST_IPV4,
        help = "IPv4 address of the service instance.",
    )]
    pub ipv4: Option<std::net::Ipv4Addr>,

    #[arg(
        long,
        env = var::HOST_IPV6,
        help = "IPv6 address of the service instance.",
    )]
    pub ipv6: Option<std::net::Ipv6Addr>,

    #[arg(
        long,
        env = var::PORT,
        default_value_t = default::PORT,
        help = "Port the service instances listens on.",
        value_parser = clap::value_parser!(u16).range(1024..),
    )]
    pub port: u16,

    #[arg(
        long,
        env = var::APP_SERVICE_PORT,
        default_value_t = default::APP_SERVICE_PORT,
        help = "Port the application service listens on.",
        value_parser = clap::value_parser!(u16).range(1024..),
    )]
    pub app_service_port: u16,

    /// Browser-visible Origin allowed by CORS. Must match the scheme/host/port the user-agent
    /// sends in its `Origin` header (i.e. how the app-service is reached, not where auth-service
    /// is bound). When unset, defaults to `http://localhost:{app_service_port}`.
    #[arg(
        long,
        env = var::ALLOWED_ORIGIN,
        help = "Browser-visible URL of the application service for CORS allow-origin.",
        value_parser,
    )]
    pub allowed_origin: Option<url::Url>,
}

impl NetworkConfig {
    /// Resolved CORS allow-origin. Falls back to `http://localhost:{app_service_port}` so local
    /// dev / Docker browsers (which connect via `localhost`) work without extra config.
    #[must_use]
    pub fn resolved_allowed_origin(&self) -> url::Url {
        self.allowed_origin.clone().unwrap_or_else(|| {
            url::Url::parse(&format!("http://localhost:{}", self.app_service_port))
                .expect("default allowed origin must be a valid URL")
        })
    }
}

impl NetworkConfig {
    #[must_use]
    pub fn from_environment() -> Self {
        let app_service_port = std::env::var(var::APP_SERVICE_PORT)
            .ok()
            .and_then(|s| s.parse::<u16>().ok())
            .filter(|&p| p >= 1024)
            .unwrap_or_else(|| {
                tracing::warn!(
                    "using default value: {}={}",
                    var::APP_SERVICE_PORT,
                    default::APP_SERVICE_PORT,
                );
                default::APP_SERVICE_PORT
            });

        let mut ipv4 = std::env::var(var::HOST_IPV4)
            .ok()
            .and_then(|s| s.parse::<std::net::Ipv4Addr>().ok());

        let mut ipv6 = std::env::var(var::HOST_IPV6)
            .ok()
            .and_then(|s| s.parse::<std::net::Ipv6Addr>().ok());

        if ipv4.is_none() && ipv6.is_none() {
            ipv4 = default::HOST_IPV4;
            tracing::warn!("both {} and {} are not set", var::HOST_IPV4, var::HOST_IPV6,);
            tracing::warn!("using IPv4 default value: {}={:?}", var::HOST_IPV4, default::HOST_IPV4,);
        }

        if ipv4.is_some() && ipv6.is_some() {
            ipv6 = None;
            tracing::warn!("both {} and {} are set", var::HOST_IPV4, var::HOST_IPV6,);
            tracing::warn!("invalidating IPv6");
        }

        let port = std::env::var(var::PORT)
            .ok()
            .and_then(|s| s.parse::<u16>().ok())
            .filter(|&p| p >= 1024)
            .unwrap_or_else(|| {
                tracing::warn!("using default value: {}={}", var::PORT, default::PORT,);
                default::PORT
            });

        let allowed_origin = std::env::var(var::ALLOWED_ORIGIN)
            .ok()
            .and_then(|s| s.parse::<url::Url>().ok());

        Self {
            app_service_port,
            ipv4,
            ipv6,
            port,
            allowed_origin,
        }
    }
}

impl std::fmt::Display for NetworkConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NetworkConfig")
            .field("app_service_port", &self.app_service_port)
            .field("ipv4", &self.ipv4)
            .field("ipv6", &self.ipv6)
            .field("port", &self.port)
            .field("allowed_origin", &self.allowed_origin)
            .finish()
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            app_service_port: default::APP_SERVICE_PORT,
            ipv4: default::HOST_IPV4,
            ipv6: None,
            port: default::PORT,
            allowed_origin: None,
        }
    }
}
