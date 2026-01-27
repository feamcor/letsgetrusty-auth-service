use clap::ArgGroup;
use clap::Parser;
use clap::ValueEnum;
use fmt::{Display, Formatter};
use std::fmt;
use std::net::{Ipv4Addr, Ipv6Addr};

pub const CONFIG_HOST_IPV4: &str = "AUTH_SERVICE_HOST_IPV4";
pub const CONFIG_HOST_IPV6: &str = "AUTH_SERVICE_HOST_IPV6";
pub const CONFIG_PORT: &str = "AUTH_SERVICE_PORT";
pub const CONFIG_LOG: &str = "AUTH_SERVICE_LOG";

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
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Trace => write!(formatter, "trace"),
            LogLevel::Debug => write!(formatter, "debug"),
            LogLevel::Info => write!(formatter, "info"),
            LogLevel::Warn => write!(formatter, "warn"),
            LogLevel::Error => write!(formatter, "error"),
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(group(
    ArgGroup::new("ip_address_selection")
    .args(["ipv4","ipv6"])
    .required(false)
    .multiple(false)
))]
pub struct Config {
    #[arg(
        long,
        env = CONFIG_HOST_IPV4,
        help = "IPv4 address for the service to listen on.",
    )]
    pub ipv4: Option<Ipv4Addr>,
    #[arg(
        long,
        env = CONFIG_HOST_IPV6,
        help = "IPv6 address for the service to listen on.",
    )]
    pub ipv6: Option<Ipv6Addr>,
    #[arg(
        long,
        env = CONFIG_PORT,
        default_value = "3000",
        help = "Port for the service to listen on.",
        value_parser = clap::value_parser!(u16).range(1024..),
    )]
    pub port: u16,
    #[arg(
        long,
        env = CONFIG_LOG,
        default_value = "info",
        help = "Log level for the service.",
    )]
    pub log: LogLevel,
}

impl Display for Config {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Config {{ ipv4:{:?}, ipv6:{:?}, port:{:?}, log:{:?} }}",
            self.ipv4,
            self.ipv6,
            self.port,
            self.log,
        )
    }
}