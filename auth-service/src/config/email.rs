use crate::config;
use crate::domain::Email;
use crate::domain::Secret;
use clap::ValueEnum;

pub mod var {
    pub const SERVICE: &str = "AUTH_SERVICE_EMAIL_SERVICE";
    pub const API_KEY: &str = "AUTH_SERVICE_EMAIL_API_KEY";
    pub const API_TIMEOUT: &str = "AUTH_SERVICE_EMAIL_API_TIMEOUT";
    pub const API_URL: &str = "AUTH_SERVICE_EMAIL_API_URL";
    pub const SENDER: &str = "AUTH_SERVICE_EMAIL_SENDER";
    pub const STREAM: &str = "AUTH_SERVICE_EMAIL_STREAM";
}

pub mod default {
    use super::EmailService;
    pub const SERVICE: EmailService = EmailService::Mock;
    pub const API_TIMEOUT: u32 = 30000; // 30 second
    pub const API_URL: &str = "https://api.postmarkapp.com/";
    pub const STREAM: &str = "outbound";
}

#[derive(clap::ValueEnum, Clone, Debug, PartialEq)]
#[value(rename_all = "kebab-case")]
pub enum EmailService {
    Mock,
    Postmark,
}

impl std::fmt::Display for EmailService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_possible_value().unwrap().get_name())
    }
}

#[derive(clap::Args, Debug)]
pub struct EmailServiceConfig {
    #[arg(
        long,
        env = var::SERVICE,
        default_value_t = default::SERVICE,
        help = "Email service to be used for sending notifications.",
        value_parser = clap::value_parser!(EmailService),
    )]
    pub email_service: EmailService,

    #[arg(skip)]
    pub email_api_key: Option<Secret>,

    #[arg(
        long,
        env = var::API_TIMEOUT,
        default_value_t = default::API_TIMEOUT,
        help = "Email service API timeout in milliseconds.",
        value_parser = clap::value_parser!(u32).range(100..60000),
    )]
    pub email_api_timeout: u32,

    #[arg(
        long,
        env = var::API_URL,
        default_value_t = url::Url::parse(default::API_URL).unwrap(),
        help = "URL of the email service API.",
        value_parser,
    )]
    pub email_api_url: url::Url,

    #[arg(
        long,
        env = var::SENDER,
        help = "Email address to be used as sender of notifications.",
        value_parser,
    )]
    pub email_sender: Option<Email>,

    #[arg(
        long,
        env = var::STREAM,
        default_value_t = String::from(default::STREAM),
        help = "Stream used by the email service to group notifications.",
    )]
    pub email_stream: String,
}

impl EmailServiceConfig {
    #[must_use]
    pub fn from_environment() -> Self {
        let email_service = std::env::var(var::SERVICE)
            .ok()
            .and_then(|s| EmailService::from_str(&s, true).ok())
            .unwrap_or_else(|| {
                tracing::warn!("using default value: {}={}", var::SERVICE, default::SERVICE);
                default::SERVICE
            });

        let email_service_api_timeout = std::env::var(var::API_TIMEOUT)
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .filter(|&ttl| (100..60000).contains(&ttl))
            .unwrap_or_else(|| {
                tracing::warn!("using default value: {}={}", var::API_TIMEOUT, default::API_TIMEOUT,);
                default::API_TIMEOUT
            });

        let email_service_api_url = std::env::var(var::API_URL)
            .ok()
            .and_then(|s| s.parse::<url::Url>().ok())
            .unwrap_or_else(|| {
                tracing::warn!("using default value: {}={}", var::API_URL, default::API_URL,);
                default::API_URL.parse().unwrap()
            });

        let email_service_stream = std::env::var(var::STREAM).ok().unwrap_or_else(|| {
            tracing::warn!("using default value: {}={}", var::STREAM, default::STREAM,);
            default::STREAM.to_owned()
        });

        Self {
            email_service,
            email_api_key: None,
            email_api_timeout: email_service_api_timeout,
            email_api_url: email_service_api_url,
            email_sender: None,
            email_stream: email_service_stream,
        }
    }

    pub fn load_mandatory_arguments(&mut self) {
        if self.email_service == EmailService::Postmark {
            let email_api_key = config::secret_from_environment(var::API_KEY);
            let email_sender = config::secret_from_environment(var::SENDER);
            let email_sender = email_sender.as_ref();
            let email_sender = email_sender.unwrap();
            let email_sender = Email::parse(email_sender).unwrap_or_else(|e| {
                tracing::error!("{}: {}", var::SENDER, e);
                panic!("{}: {}", var::SENDER, e)
            });
            self.email_api_key = email_api_key;
            self.email_sender = Some(email_sender);
        }
    }
}

impl std::fmt::Display for EmailServiceConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EmailServiceConfig")
            .field("email_service", &self.email_service)
            .field("email_api_key", &self.email_api_key)
            .field("email_api_timeout", &self.email_api_timeout)
            .field("email_api_url", &self.email_api_url)
            .field("email_sender", &self.email_sender)
            .field("email_stream", &self.email_stream)
            .finish()
    }
}

impl Default for EmailServiceConfig {
    fn default() -> Self {
        Self {
            email_service: default::SERVICE,
            email_api_key: None,
            email_api_timeout: default::API_TIMEOUT,
            email_api_url: default::API_URL.parse().unwrap(),
            email_sender: None,
            email_stream: default::STREAM.to_owned(),
        }
    }
}
