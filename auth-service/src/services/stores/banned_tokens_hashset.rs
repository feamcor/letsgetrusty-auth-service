use crate::domain::Token;
use crate::services::BannedTokenStore;
use crate::services::BannedTokenStoreError;
use crate::services::BannedTokenStoreResult;
use std::collections::HashSet;
use tokio::sync::RwLock;

#[derive(Debug, Default)]
pub struct HashsetBannedTokenStore {
    tokens: RwLock<HashSet<String>>,
}

#[async_trait::async_trait]
impl BannedTokenStore for HashsetBannedTokenStore {
    async fn add_token(&self, token: &Token) -> BannedTokenStoreResult<()> {
        // Hold the write lock for the whole check-and-insert so concurrent add_token calls can't
        // both observe "absent" and race to insert (matches the Redis variant's NX semantics).
        let token = token.as_secret().expose().to_owned();
        let mut tokens = self.tokens.write().await;
        if tokens.insert(token) {
            Ok(())
        } else {
            Err(BannedTokenStoreError::TokenAlreadyExists)
        }
    }

    async fn is_token_banned(&self, token: &Token) -> BannedTokenStoreResult<bool> {
        let token = token.as_secret().expose();
        Ok(self.tokens.read().await.contains(token))
    }

    async fn remove_token(&self, token: &Token) -> BannedTokenStoreResult<()> {
        let token = token.as_secret().expose();
        if self.tokens.write().await.remove(token) {
            Ok(())
        } else {
            Err(BannedTokenStoreError::TokenNotFound)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_banned_token() {
        let store = HashsetBannedTokenStore::default();
        let token = Token::new(&"test_token".into());
        assert!(store.add_token(&token).await.is_ok());
        assert!(store.is_token_banned(&token).await.unwrap());
        let result = store.add_token(&token).await;
        assert!(result.is_err());
        assert!(match result.unwrap_err() {
            BannedTokenStoreError::TokenAlreadyExists => true,
            _ => panic!("expected TokenAlreadyExists error"),
        });
    }

    #[tokio::test]
    async fn test_is_token_banned() {
        let store = HashsetBannedTokenStore::default();
        let token = Token::new(&"test_token".into());
        assert!(!store.is_token_banned(&token).await.unwrap());
        store.add_token(&token).await.unwrap();
        assert!(store.is_token_banned(&token).await.unwrap());
        let other_token = Token::new(&"other_token".into());
        assert!(!store.is_token_banned(&other_token).await.unwrap());
    }

    #[tokio::test]
    async fn test_remove_banned_token() {
        let store = HashsetBannedTokenStore::default();
        let token = Token::new(&"test_token".into());
        store.add_token(&token).await.unwrap();
        assert!(store.is_token_banned(&token).await.unwrap());
        assert!(store.remove_token(&token).await.is_ok());
        assert!(!store.is_token_banned(&token).await.unwrap());
        let result = store.remove_token(&token).await;
        assert!(result.is_err());
        assert!(match result.unwrap_err() {
            BannedTokenStoreError::TokenNotFound => true,
            _ => panic!("Expected TokenNotFound error"),
        });
    }
}
