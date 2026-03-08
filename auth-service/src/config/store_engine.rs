use clap::ValueEnum;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(ValueEnum, Clone, Debug)]
#[value(rename_all = "kebab-case")]
pub enum StoreEngine {
    Ephemeral,
    Server,
}

impl Display for StoreEngine {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}",
            self.to_possible_value().unwrap().get_name()
        )
    }
}
