use crate::config::consts;
use crate::services::{BannedTokenStore, BannedTokenStoreError};
use redis::{Commands, ExistenceCheck, SetExpiry, SetOptions};
use std::fmt::Debug;
use tokio::sync::RwLock;


#[allow(unused_imports)]
use tracing::Level;

pub struct RedisBannedTokenStore {
    connection: RwLock<redis::Connection>,
}

impl Debug for RedisBannedTokenStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisBannedTokenStore")
            .field("connection", &"RwLock<redis::Connection>")
            .finish()
    }
}

impl RedisBannedTokenStore {
    pub fn new(connection: RwLock<redis::Connection>) -> Self {
        Self { connection }
    }
}

#[async_trait::async_trait]
impl BannedTokenStore for RedisBannedTokenStore {
    #[tracing::instrument(name = "AddBannedTokenIntoCache", level = Level::TRACE, skip_all)]
    async fn add_token(&self, token: &str) -> Result<(), BannedTokenStoreError> {
        let ttl = u64::from(consts::AUTH_SERVICE_JWT_TTL_SECONDS_DEFAULT);
        let key = token_key(token);
        let mut connection = self.connection.write().await;
        let options = SetOptions::default()
            .conditional_set(ExistenceCheck::NX)
            .with_expiration(SetExpiry::EX(ttl));
        let result: redis::RedisResult<Option<String>> = connection.set_options(key, true, options);
        match result {
            Ok(Some(_)) => Ok(()),
            Ok(None) => Err(BannedTokenStoreError::TokenAlreadyExists(token.to_string())),
            Err(error) => Err(BannedTokenStoreError::UnexpectedError(error.into())),
        }
    }

    #[tracing::instrument(name = "CheckBannedTokenInCache", level = Level::TRACE, skip_all)]
    async fn is_token_banned(&self, token: &str) -> Result<bool, BannedTokenStoreError> {
        let key = token_key(token);
        let mut connection = self.connection.write().await;
        connection
            .exists(key)
            .map_err(|error| BannedTokenStoreError::UnexpectedError(error.into()))
    }

    #[tracing::instrument(name = "RemoveBannedTokenFromCache", level = Level::TRACE, skip_all)]
    async fn remove_token(&self, token: &str) -> Result<(), BannedTokenStoreError> {
        let key = token_key(token);
        let mut connection = self.connection.write().await;
        connection
            .del(key)
            .map_err(|error| BannedTokenStoreError::UnexpectedError(error.into()))
    }
}

fn token_key(token: &str) -> String {
    format!("token:banned:{token}")
}
