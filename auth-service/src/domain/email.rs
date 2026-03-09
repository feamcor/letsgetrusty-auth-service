use email_address::{EmailAddress, Options};
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Email(EmailAddress);

#[derive(thiserror::Error, Debug)]
#[error("Invalid email: {0}")]
pub struct EmailError(String);

pub const EMAIL_OPTIONS: Options = Options {
    minimum_sub_domains: 2,
    allow_domain_literal: false,
    allow_display_text: false,
};

impl Email {
    pub fn parse(email: &str) -> Result<Self, EmailError> {
        EmailAddress::parse_with_options(email, EMAIL_OPTIONS)
            .map(Self)
            .map_err(|error| EmailError(error.to_string()))
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;

    #[test]
    fn should_parse_valid_email() {
        let email: String = SafeEmail().fake();
        assert!(Email::parse(&email).is_ok());
    }

    #[test]
    fn should_reject_invalid_email() {
        let email = "invalid-email";
        assert!(Email::parse(email).is_err());
    }
}
