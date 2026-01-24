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
    #[error("Unexpected error")]
    UnexpectedError,
}

impl User {
    pub fn try_new(email: &str, password: &str, requires_2fa: bool) -> Result<Self, UserError> {
        let options = Options {
            minimum_sub_domains: 2,
            allow_domain_literal: false,
            allow_display_text: false,
        };
        let email = EmailAddress::parse_with_options(email, options)
            .map_err(UserError::InvalidEmail)?;
        let password = Password::parse(password)
            .map_err(UserError::InvalidPassword)?;
        Ok(Self { email, password, requires_2fa })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn should_return_ok() {
        todo!()
    }

    #[test]
    fn should_return_invalid_email() {
        todo!()
    }

    #[test]
    fn should_return_invalid_password() {
        todo!()
    }
}