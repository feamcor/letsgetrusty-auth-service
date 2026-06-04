use crate::domain::Email;
use crate::domain::User;
use crate::services::UserStore;
use crate::services::UserStoreError;
use crate::services::UserStoreResult;
use std::collections::HashMap;
use tokio::sync::RwLock;

/// In-memory [`UserStore`] keyed by email; for development and tests (state is lost on restart).
#[derive(Debug, Default)]
pub struct HashmapUserStore {
    users: RwLock<HashMap<String, User>>,
}

#[async_trait::async_trait]
impl UserStore for HashmapUserStore {
    #[allow(clippy::map_entry)]
    async fn add_user(&self, user: User) -> UserStoreResult<()> {
        let email = user.email.as_secret().expose().to_owned();
        let mut users = self.users.write().await;
        if users.contains_key(&email) {
            Err(UserStoreError::UserAlreadyExists)
        } else {
            users.insert(email, user);
            Ok(())
        }
    }

    async fn get_user(&self, email: &Email) -> UserStoreResult<User> {
        self.users
            .read()
            .await
            .get(email.as_secret().expose())
            .cloned()
            .ok_or(UserStoreError::UserNotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::HashedPassword;

    #[tokio::test]
    async fn test_add_user() {
        let email = Email::parse(&"alice@example.com".into()).unwrap();
        let password = HashedPassword::parse(&"StrongPassword123!".into(), &email)
            .await
            .unwrap();
        let user_1 = User::new(&email, &password, false);
        let user_2 = user_1.clone();
        let store = HashmapUserStore::default();
        assert!(store.add_user(user_1).await.is_ok());
        assert!(store.add_user(user_2).await.is_err());
    }

    #[tokio::test]
    async fn test_get_user() {
        let email = Email::parse(&"alice@example.com".into()).unwrap();
        let password = HashedPassword::parse(&"StrongPassword123!".into(), &email)
            .await
            .unwrap();
        let user = User::new(&email, &password, false);
        let store = HashmapUserStore::default();
        store.add_user(user).await.unwrap();
        assert!(store.get_user(&email).await.is_ok());
        let email = Email::parse(&"bob@example.com".into()).unwrap();
        assert!(store.get_user(&email).await.is_err());
    }

    #[tokio::test]
    async fn test_validate_user() {
        let email = Email::parse(&"alice@example.com".into()).unwrap();
        let password = HashedPassword::parse(&"StrongPassword123!".into(), &email)
            .await
            .unwrap();
        let user = User::new(&email, &password, false);
        let store = HashmapUserStore::default();
        store.add_user(user).await.unwrap();
        assert!(store.validate_user(&email, &"StrongPassword123!".into()).await.is_ok());
        assert!(store.validate_user(&email, &"StrongPassword456!".into()).await.is_err());
    }
}
