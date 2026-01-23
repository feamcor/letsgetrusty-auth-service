use crate::domain::User;
use crate::services::{UserStore, UserStoreError};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct HashmapUserStore {
    users: HashMap<String, User>,
}

#[async_trait::async_trait]
impl UserStore for HashmapUserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError> {
        let email = user.email.to_string();
        if self.users.contains_key(&email) {
            Err(UserStoreError::UserAlreadyExists(email))
        } else {
            self.users.insert(email, user);
            Ok(())
        }
    }

    async fn get_user(&self, email: &str) -> Result<&User, UserStoreError> {
        self.users
            .get(email)
            .ok_or(UserStoreError::UserNotFound(email.to_string()))
    }

    async fn validate_user(&self, email: &str, password: &str) -> Result<(), UserStoreError> {
        let user = self.get_user(email).await?;
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
        assert!(store.add_user(user_1).await.is_ok());
        assert!(store.add_user(user_2).await.is_err());
    }

    #[tokio::test]
    async fn test_get_user() {
        let user = User::try_new(
            "alice@example.com",
            "StrongPassword123!",
            false).unwrap();
        let mut store = HashmapUserStore::default();
        store.add_user(user).await.unwrap();
        assert!(store.get_user("alice@example.com").await.is_ok());
        assert!(store.get_user("bob@example.com").await.is_err());
    }

    #[tokio::test]
    async fn test_validate_user() {
        let user = User::try_new(
            "alice@example.com",
            "StrongPassword123!",
            false).unwrap();
        let mut store = HashmapUserStore::default();
        store.add_user(user).await.unwrap();
        assert!(store.validate_user("alice@example.com", "StrongPassword123!").await.is_ok());
        assert!(store.validate_user("alice@example.com", "StrongPassword456!").await.is_err());
    }
}
