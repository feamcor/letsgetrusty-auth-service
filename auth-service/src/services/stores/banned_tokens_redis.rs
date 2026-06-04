use crate::domain::Token;
use crate::services::BannedTokenStore;
use crate::services::BannedTokenStoreError;
use crate::services::BannedTokenStoreResult;
use redis::AsyncCommands;
use redis::ExistenceCheck;
use redis::SetExpiry;
use redis::SetOptions;
use std::fmt::Debug;

/// Redis-backed [`BannedTokenStore`] with per-entry expiry (the production cache backend).
pub struct RedisBannedTokenStore {
    connection: redis::aio::MultiplexedConnection,
    jwt_ttl_secs: u64,
}

impl Debug for RedisBannedTokenStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisBannedTokenStore")
            .field("connection", &"MultiplexedConnection")
            .field("jwt_ttl_secs", &self.jwt_ttl_secs)
            .finish()
    }
}

impl RedisBannedTokenStore {
    pub fn new(connection: redis::aio::MultiplexedConnection, jwt_ttl_secs: u64) -> Self {
        Self {
            connection,
            jwt_ttl_secs,
        }
    }
}

#[async_trait::async_trait]
impl BannedTokenStore for RedisBannedTokenStore {
    #[tracing::instrument(name = "AddBannedTokenIntoCache", level = tracing::Level::TRACE, skip_all)]
    async fn add_token(&self, token: &Token) -> BannedTokenStoreResult<()> {
        let key = token_key(token);
        let mut connection = self.connection.clone();
        let options = SetOptions::default()
            .conditional_set(ExistenceCheck::NX)
            .with_expiration(SetExpiry::EX(self.jwt_ttl_secs));
        let result: redis::RedisResult<Option<String>> = connection.set_options(key, true, options).await;
        match result {
            Ok(Some(_)) => Ok(()),
            Ok(None) => Err(BannedTokenStoreError::TokenAlreadyExists),
            Err(error) => Err(BannedTokenStoreError::UnexpectedError(error.into())),
        }
    }

    #[tracing::instrument(name = "CheckBannedTokenInCache", level = tracing::Level::TRACE, skip_all)]
    async fn is_token_banned(&self, token: &Token) -> BannedTokenStoreResult<bool> {
        let key = token_key(token);
        let mut connection = self.connection.clone();
        connection
            .exists(key)
            .await
            .map_err(|error| BannedTokenStoreError::UnexpectedError(error.into()))
    }

    #[tracing::instrument(name = "RemoveBannedTokenFromCache", level = tracing::Level::TRACE, skip_all)]
    async fn remove_token(&self, token: &Token) -> BannedTokenStoreResult<()> {
        let key = token_key(token);
        let mut connection = self.connection.clone();
        connection
            .del(key)
            .await
            .map_err(|error| BannedTokenStoreError::UnexpectedError(error.into()))
    }
}

fn token_key(token: &Token) -> String {
    let raw_token = token.as_secret().expose();
    format!("token:banned:{raw_token}")
}
