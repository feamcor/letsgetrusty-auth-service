use crate::domain::{Password, PasswordError};
use email_address::{EmailAddress, Options};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct User {
    pub email: EmailAddress,
    pub password: Password,
    pub requires_2fa: bool,
}

#[derive(Error, Debug)]
pub enum UserError {
    #[error("Invalid email: {0}")]
    InvalidEmail(email_address::Error),
    #[error("Invalid password: {0}")]
    InvalidPassword(PasswordError),
}

impl User {
    pub fn try_new(email: &str, password: &str, requires_2fa: bool) -> Result<Self, UserError> {
        let options = Options {
            minimum_sub_domains: 2,
            allow_domain_literal: false,
            allow_display_text: false,
        };
        let email_address = EmailAddress::parse_with_options(email, options)
            .map_err(UserError::InvalidEmail)?;
        let password = Password::parse(password, email)
            .map_err(UserError::InvalidPassword)?;
        Ok(Self { email: email_address, password, requires_2fa })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::faker::internet::en::SafeEmail;
    use fake::{rand, Fake};

    const VALID_PASSWORD: &str = "StrongPassword123!";
    const INVALID_PASSWORD: &str = "Weak!";

    #[test]
    fn should_return_ok_for_valid_input() {
        let email: String = SafeEmail().fake();
        let password = VALID_PASSWORD;
        let requires_2fa = rand::random();
        let result = User::try_new(&email, password, requires_2fa);
        assert!(result.is_ok(), "Failed for email: {} and password: {}", email, password);
    }

    #[test]
    fn should_return_error_for_empty_email() {
        let email = "";
        let password = VALID_PASSWORD;
        let requires_2fa = rand::random();
        let result = User::try_new(email, &password, requires_2fa);
        assert!(matches!(result, Err(UserError::InvalidEmail(_))));
    }

    #[test]
    fn should_return_error_for_empty_password() {
        let email: String = SafeEmail().fake();
        let password = "";
        let requires_2fa = rand::random();
        let result = User::try_new(&email, password, requires_2fa);
        assert!(matches!(result, Err(UserError::InvalidPassword(_))));
    }

    #[test]
    fn should_return_invalid_email_error() {
        let email = "invalid-email";
        let password = VALID_PASSWORD;
        let requires_2fa = rand::random();
        let result = User::try_new(email, password, requires_2fa);
        assert!(matches!(result, Err(UserError::InvalidEmail(_))));
    }

    #[test]
    fn should_return_invalid_password_error() {
        let email: String = SafeEmail().fake();
        let password = INVALID_PASSWORD;
        let requires_2fa = rand::random();
        let result = User::try_new(&email, password, requires_2fa);
        assert!(matches!(result, Err(UserError::InvalidPassword(_))));
    }
}
