use crate::domain::{Password, PasswordError};
use email_address::EmailAddress;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct User {
    pub email: EmailAddress,
    pub password: Password,
    pub requires_2fa: bool,
}

#[derive(Error, Debug)]
pub enum UserError {
    #[error("Invalid email address: {0}")]
    InvalidEmailAddress(email_address::Error),
    #[error("Invalid password: {0}")]
    InvalidPassword(PasswordError),
    #[error("Unexpected error")]
    UnexpectedError,
}

impl User {
    pub fn try_new(email: &str, password: &str, requires_2fa: bool) -> Result<Self, UserError> {
        let email = EmailAddress::from_str(email)
            .map_err(UserError::InvalidEmailAddress)?;
        let password = Password::parse(password)
            .map_err(UserError::InvalidPassword)?;
        Ok(Self { email, password, requires_2fa })
    }
}