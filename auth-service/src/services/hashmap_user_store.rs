use crate::domain::User;
use std::collections::HashMap;
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

#[derive(Debug, Default)]
pub struct HashmapUserStore {
    users: HashMap<String, User>,
}

impl HashmapUserStore {
    pub fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        let email = user.email.to_string();
        if self.users.contains_key(&email) {
            Err(UserStoreError::UserAlreadyExists(email))
        } else {
            self.users.insert(email, user);
            Ok(())
        }
    }

    pub fn get_user(&self, email: &str) -> Result<&User, UserStoreError> {
        self.users
            .get(email)
            .ok_or(UserStoreError::UserNotFound(email.to_string()))
    }

    pub fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError> {
        let user = self.get_user(email)?;
        if user.password.expose() == password {
            Ok(())
        } else {
            Err(UserStoreError::InvalidCredentials(email.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_user() {
        let user_1 = User::try_new(
            "alice@example.com",
            "StrongPassword123!",
            false).unwrap();
        let user_2 = user_1.clone();
        let mut store = HashmapUserStore::default();
        assert!(store.add_user(user_1).is_ok());
        assert!(store.add_user(user_2).is_err());
    }

    #[tokio::test]
    async fn test_get_user() {
        let user = User::try_new(
            "alice@example.com",
            "StrongPassword123!",
            false).unwrap();
        let mut store = HashmapUserStore::default();
        store.add_user(user).unwrap();
        assert!(store.get_user("alice@example.com").is_ok());
        assert!(store.get_user("bob@example.com").is_err());
    }

    #[tokio::test]
    async fn test_validate_user() {
        let user = User::try_new(
            "alice@example.com",
            "StrongPassword123!",
            false).unwrap();
        let mut store = HashmapUserStore::default();
        store.add_user(user).unwrap();
        assert!(store.validate_user("alice@example.com", "StrongPassword123!").is_ok());
        assert!(store.validate_user("alice@example.com", "StrongPassword456!").is_err());
    }
}
