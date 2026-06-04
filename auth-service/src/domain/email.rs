use crate::domain::Secret;
use email_address::EmailAddress;
use email_address::Options;
use std::str::FromStr;

/// A syntactically valid, lowercase-normalized email address.
///
/// Construct via [`Email::parse`]; the inner value is a redacted [`Secret`], so the address never
/// leaks through `Debug`/logs. Normalization makes case-variant addresses compare and hash equal.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Email(Secret);

impl Email {
    /// Parse and normalize an email address (trimmed, lowercased, RFC-validated).
    ///
    /// # Errors
    ///
    /// Returns [`EmailError`] if the input is not a valid address (requires at least two
    /// sub-domains; rejects domain literals and display text).
    pub fn parse(email: &Secret) -> EmailResult<Self> {
        const EMAIL_OPTIONS: Options = Options {
            minimum_sub_domains: 2,
            allow_domain_literal: false,
            allow_display_text: false,
        };
        // Normalize to lowercase so equivalent addresses ("Alice@Example.com" vs
        // "alice@example.com") hash and compare as the same identity.
        let normalized: Secret = email.expose().trim().to_lowercase().into();
        match EmailAddress::parse_with_options(normalized.expose(), EMAIL_OPTIONS) {
            Ok(_) => Ok(Self(normalized)),
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

// this will allow clap to validate emails passed as command line arguments
impl FromStr for Email {
    type Err = EmailError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let email = Secret::from(s);
        Self::parse(&email)
    }
}

/// Error returned when an email address fails validation.
#[derive(thiserror::Error, Debug)]
#[error("Invalid email")]
pub struct EmailError(#[from] color_eyre::eyre::Report);

/// Convenience alias for a fallible [`Email`] operation.
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

    #[test]
    fn parse_lowercases_input() {
        let parsed = Email::parse(&"Alice@Example.com".into()).unwrap();
        assert_eq!(parsed.as_secret().expose(), "alice@example.com");
    }

    #[test]
    fn parse_trims_surrounding_whitespace() {
        let parsed = Email::parse(&"  alice@example.com  ".into()).unwrap();
        assert_eq!(parsed.as_secret().expose(), "alice@example.com");
    }

    #[test]
    fn case_variant_emails_are_equal_after_parse() {
        let a = Email::parse(&"Alice@Example.com".into()).unwrap();
        let b = Email::parse(&"ALICE@example.COM".into()).unwrap();
        assert_eq!(a, b);
    }
}
