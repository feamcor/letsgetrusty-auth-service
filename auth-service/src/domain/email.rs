use crate::domain::Secret;
use email_address::EmailAddress;
use email_address::Options;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Email(Secret);

impl Email {
    pub fn parse(email: &Secret) -> EmailResult<Self> {
        const EMAIL_OPTIONS: Options = Options {
            minimum_sub_domains: 2,
            allow_domain_literal: false,
            allow_display_text: false,
        };
        match EmailAddress::parse_with_options(email.expose(), EMAIL_OPTIONS) {
            Ok(_) => Ok(Self(email.to_owned())),
            Err(error) => Err(EmailError(error.into())),
        }
    }

    #[must_use]
    pub fn as_secret(&self) -> &Secret {
        &self.0
    }
}

impl PartialEq for Email {
    fn eq(&self, other: &Self) -> bool {
        self.as_secret() == other.as_secret()
    }
}

impl Eq for Email {}

impl std::hash::Hash for Email {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_secret().hash(state);
    }
}

#[derive(thiserror::Error, Debug)]
#[error("Invalid email")]
pub struct EmailError(#[from] color_eyre::eyre::Report);

pub type EmailResult<T> = Result<T, EmailError>;

#[cfg(test)]
mod tests {
    use super::*;
    use fake::Fake;
    use fake::faker::internet::en::SafeEmail;

    #[test]
    fn should_parse_valid_email() {
        let email = SafeEmail().fake::<String>().into();
        assert!(Email::parse(&email).is_ok());
    }

    #[test]
    fn should_reject_invalid_email() {
        let email = "invalid-email".into();
        assert!(Email::parse(&email).is_err());
    }
}
