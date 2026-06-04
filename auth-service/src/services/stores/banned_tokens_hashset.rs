use crate::domain::Token;
use crate::services::BannedTokenStore;
use crate::services::BannedTokenStoreError;
use crate::services::BannedTokenStoreResult;
use std::collections::HashMap;
use std::time::Duration;
use std::time::Instant;
use tokio::sync::RwLock;

/// In-memory [`BannedTokenStore`] with TTL-based expiry; for development and tests.
#[derive(Debug)]
pub struct HashsetBannedTokenStore {
    tokens: RwLock<HashMap<String, Instant>>,
    ttl: Duration,
}

impl HashsetBannedTokenStore {
    /// Construct with the JWT TTL — entries older than this are treated as expired and swept on
    /// the next `is_token_banned` / `add_token` call. Mirrors `RedisBannedTokenStore::new`.
    #[must_use]
    pub fn new(jwt_ttl_secs: u64) -> Self {
        Self {
            tokens: RwLock::default(),
            ttl: Duration::from_secs(jwt_ttl_secs),
        }
    }
}

#[async_trait::async_trait]
impl BannedTokenStore for HashsetBannedTokenStore {
    async fn add_token(&self, token: &Token) -> BannedTokenStoreResult<()> {
        // Hold the write lock for the whole check-and-insert so concurrent add_token calls can't
        // both observe "absent" and race to insert (matches the Redis variant's NX semantics).
        let token = token.as_secret().expose().to_owned();
        let mut tokens = self.tokens.write().await;
        // Drop the existing entry if it's already expired so the operator-perceived ban list
        // doesn't bloat after long uptime.
        if let Some(stored_at) = tokens.get(&token)
            && stored_at.elapsed() >= self.ttl
        {
            tokens.remove(&token);
        }
        if tokens.insert(token, Instant::now()).is_none() {
            Ok(())
        } else {
            Err(BannedTokenStoreError::TokenAlreadyExists)
        }
    }

    async fn is_token_banned(&self, token: &Token) -> BannedTokenStoreResult<bool> {
        let raw = token.as_secret().expose();
        // Read under a read-lock first to keep the hot path lock-free of writes.
        {
            let tokens = self.tokens.read().await;
            if let Some(stored_at) = tokens.get(raw) {
                if stored_at.elapsed() < self.ttl {
                    return Ok(true);
                }
                // Fall through to sweep below.
            } else {
                return Ok(false);
            }
        }
        // Expired entry observed — sweep, but re-check under the write lock so a concurrent
        // re-ban isn't blindly evicted.
        let mut tokens = self.tokens.write().await;
        match tokens.get(raw) {
            Some(stored_at) if stored_at.elapsed() < self.ttl => Ok(true),
            Some(_) => {
                tokens.remove(raw);
                Ok(false)
            }
            None => Ok(false),
        }
    }

    async fn remove_token(&self, token: &Token) -> BannedTokenStoreResult<()> {
        let token = token.as_secret().expose();
        if self.tokens.write().await.remove(token).is_some() {
            Ok(())
        } else {
            Err(BannedTokenStoreError::TokenNotFound)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> HashsetBannedTokenStore {
        HashsetBannedTokenStore::new(900)
    }

    #[tokio::test]
    async fn test_add_banned_token() {
        let store = store();
        let token = Token::new(&"test_token".into());
        assert!(store.add_token(&token).await.is_ok());
        assert!(store.is_token_banned(&token).await.unwrap());
        let result = store.add_token(&token).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BannedTokenStoreError::TokenAlreadyExists));
    }

    #[tokio::test]
    async fn test_is_token_banned() {
        let store = store();
        let token = Token::new(&"test_token".into());
        assert!(!store.is_token_banned(&token).await.unwrap());
        store.add_token(&token).await.unwrap();
        assert!(store.is_token_banned(&token).await.unwrap());
        let other_token = Token::new(&"other_token".into());
        assert!(!store.is_token_banned(&other_token).await.unwrap());
    }

    #[tokio::test]
    async fn test_remove_banned_token() {
        let store = store();
        let token = Token::new(&"test_token".into());
        store.add_token(&token).await.unwrap();
        assert!(store.is_token_banned(&token).await.unwrap());
        assert!(store.remove_token(&token).await.is_ok());
        assert!(!store.is_token_banned(&token).await.unwrap());
        let result = store.remove_token(&token).await;
        assert!(matches!(result.unwrap_err(), BannedTokenStoreError::TokenNotFound));
    }

    #[tokio::test]
    async fn expired_token_is_treated_as_unbanned() {
        // 0-second TTL: any positive elapsed time means the entry has expired.
        let store = HashsetBannedTokenStore::new(0);
        let token = Token::new(&"test_token".into());
        store.add_token(&token).await.unwrap();
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert!(!store.is_token_banned(&token).await.unwrap());
        // Re-banning the same token must succeed because the expired entry was swept.
        assert!(store.add_token(&token).await.is_ok());
    }
}
