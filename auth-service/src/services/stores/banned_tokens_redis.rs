use crate::services::BannedTokenStore;
use crate::services::BannedTokenStoreError;
use crate::services::BannedTokenStoreResult;
use redis::Commands;
use redis::ExistenceCheck;
use redis::SetExpiry;
use redis::SetOptions;
use std::fmt::Debug;
use tokio::sync::RwLock;

use crate::domain::Token;
#[allow(unused_imports)]
use tracing::Level;

pub struct RedisBannedTokenStore {
    connection: RwLock<redis::Connection>,
    jwt_ttl_secs: u64,
}

impl Debug for RedisBannedTokenStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisBannedTokenStore")
            .field("connection", &"RwLock<redis::Connection>")
            .finish()
    }
}

impl RedisBannedTokenStore {
    pub fn new(connection: RwLock<redis::Connection>, jwt_ttl_secs: u64) -> Self {
        Self {
            connection,
            jwt_ttl_secs,
        }
    }
}

#[async_trait::async_trait]
impl BannedTokenStore for RedisBannedTokenStore {
    #[tracing::instrument(name = "AddBannedTokenIntoCache", level = Level::TRACE, skip_all)]
    async fn add_token(&self, token: &Token) -> BannedTokenStoreResult<()> {
        let ttl = u64::from(self.jwt_ttl_secs);
        let key = token_key(token);
        let mut connection = self.connection.write().await;
        let options = SetOptions::default()
            .conditional_set(ExistenceCheck::NX)
            .with_expiration(SetExpiry::EX(ttl));
        let result: redis::RedisResult<Option<String>> = connection.set_options(key, true, options);
        match result {
            Ok(Some(_)) => Ok(()),
            Ok(None) => Err(BannedTokenStoreError::TokenAlreadyExists),
            Err(error) => Err(BannedTokenStoreError::UnexpectedError(error.into())),
        }
    }

    #[tracing::instrument(name = "CheckBannedTokenInCache", level = Level::TRACE, skip_all)]
    async fn is_token_banned(&self, token: &Token) -> BannedTokenStoreResult<bool> {
        let key = token_key(token);
        let mut connection = self.connection.write().await;
        connection
            .exists(key)
            .map_err(|error| BannedTokenStoreError::UnexpectedError(error.into()))
    }

    #[tracing::instrument(name = "RemoveBannedTokenFromCache", level = Level::TRACE, skip_all)]
    async fn remove_token(&self, token: &Token) -> BannedTokenStoreResult<()> {
        let key = token_key(token);
        let mut connection = self.connection.write().await;
        connection
            .del(key)
            .map_err(|error| BannedTokenStoreError::UnexpectedError(error.into()))
    }
}

fn token_key(token: &Token) -> String {
    let raw_token = token.as_secret().expose();
    format!("token:banned:{raw_token}")
}
