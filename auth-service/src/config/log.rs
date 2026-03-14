use clap::ValueEnum;
pub mod var {
    pub const LEVEL: &str = "AUTH_SERVICE_LOG_LEVEL";
}

pub mod default {
    use super::LogLevel;
    pub const LEVEL: LogLevel = LogLevel::Info;
}

#[derive(clap::ValueEnum, Clone, Debug, PartialEq)]
#[value(rename_all = "kebab-case")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_possible_value().unwrap().get_name())
    }
}

impl From<&LogLevel> for tracing::Level {
    fn from(log_level: &LogLevel) -> Self {
        match log_level {
            LogLevel::Trace => tracing::Level::TRACE,
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Warn => tracing::Level::WARN,
            LogLevel::Error => tracing::Level::ERROR,
        }
    }
}

impl From<&LogLevel> for tracing::metadata::LevelFilter {
    fn from(log_level: &LogLevel) -> Self {
        tracing::Level::from(log_level).into()
    }
}

#[derive(clap::Args, Debug)]
pub struct LogConfig {
    #[arg(
        long,
        env = var::LEVEL,
        default_value_t = default::LEVEL,
        help = "Level of logging emitted by the service.",
        value_parser = clap::value_parser!(LogLevel),
    )]
    pub level: LogLevel,
}

impl LogConfig {
    #[must_use]
    pub fn from_environment() -> Self {
        let level = std::env::var(var::LEVEL)
            .ok()
            .and_then(|s| LogLevel::from_str(&s, true).ok())
            .unwrap_or_else(|| {
                tracing::warn!("using default value: {}={}", var::LEVEL, default::LEVEL,);
                default::LEVEL
            });
        Self { level }
    }
}

impl std::fmt::Display for LogConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LogConfig").field("level", &self.level).finish()
    }
}

impl Default for LogConfig {
    fn default() -> Self {
        Self { level: default::LEVEL }
    }
}
