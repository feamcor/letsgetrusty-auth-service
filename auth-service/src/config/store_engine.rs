use clap::ValueEnum;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;

#[derive(ValueEnum, Clone, Debug)]
#[value(rename_all = "kebab-case")]
pub enum StoreEngine {
    Ephemeral,
    Server,
}

impl Display for StoreEngine {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_possible_value().unwrap().get_name())
    }
}
