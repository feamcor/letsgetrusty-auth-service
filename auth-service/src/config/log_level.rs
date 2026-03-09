use clap::ValueEnum;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(ValueEnum, Clone, Debug)]
#[value(rename_all = "kebab-case")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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