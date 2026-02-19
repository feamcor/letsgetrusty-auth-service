use crate::services::{BannedTokenStore, BannedTokenStoreError};
use std::collections::HashSet;
use tokio::sync::RwLock;

#[derive(Debug, Default)]
pub struct HashsetBannedTokenStore {
    tokens: RwLock<HashSet<String>>,
}

#[async_trait::async_trait]
impl BannedTokenStore for HashsetBannedTokenStore {
    async fn add_token(&self, token: &str) -> Result<(), BannedTokenStoreError> {
        if self.tokens.read().await.contains(token) {
            Err(BannedTokenStoreError::TokenAlreadyExists(token.to_string()))
        } else {
            self.tokens.write().await.insert(token.to_string());
            Ok(())
        }
    }

    async fn is_token_banned(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        Ok(self.tokens.read().await.contains(token))
    }

    async fn remove_token(&self, token: &str) -> Result<(), BannedTokenStoreError> {
        if self.tokens.write().await.remove(token) {
            Ok(())
        } else {
            Err(BannedTokenStoreError::TokenNotFound(token.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_banned_token() {
        let store = HashsetBannedTokenStore::default();
        let token = "test_token";

        assert!(store.add_token(token).await.is_ok());
        assert!(store.is_token_banned(token).await.unwrap());

        let result = store.add_token(token).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            BannedTokenStoreError::TokenAlreadyExists(t) => assert_eq!(t, token),
            _ => panic!("Expected TokenAlreadyExists error"),
        }
    }

    #[tokio::test]
    async fn test_is_token_banned() {
        let store = HashsetBannedTokenStore::default();
        let token = "test_token";

        assert!(!store.is_token_banned(token).await.unwrap());
        store.add_token(token).await.unwrap();
        assert!(store.is_token_banned(token).await.unwrap());
        assert!(!store.is_token_banned("other_token").await.unwrap());
    }

    #[tokio::test]
    async fn test_remove_banned_token() {
        let store = HashsetBannedTokenStore::default();
        let token = "test_token";

        store.add_token(token).await.unwrap();
        assert!(store.is_token_banned(token).await.unwrap());

        assert!(store.remove_token(token).await.is_ok());
        assert!(!store.is_token_banned(token).await.unwrap());

        let result = store.remove_token(token).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            BannedTokenStoreError::TokenNotFound(t) => assert_eq!(t, token),
            _ => panic!("Expected TokenNotFound error"),
        }
    }
}
