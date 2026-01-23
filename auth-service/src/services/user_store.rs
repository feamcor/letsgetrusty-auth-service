use crate::domain::User;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UserStoreError {
    #[error("User {0} already exists")]
    UserAlreadyExists(String),
    #[error("User {0} was not found")]
    UserNotFound(String),
    #[error("User {0} invalid credentials")]
    InvalidCredentials(String),
    #[error("Unexpected error")]
    UnexpectedError,
}

pub trait UserStore {
    fn add_user(&mut self, user: User) -> Result<(), UserStoreError>;
    fn get_user(&self, email: &str) -> Result<&User, UserStoreError>;
    fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError>;
}
