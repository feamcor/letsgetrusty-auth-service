#[derive(thiserror::Error, Debug)]
pub enum BannedTokenStoreError {
    #[error("Token already exists: {0}")]
    TokenAlreadyExists(String),
    #[error("Token was not found: {0}")]
    TokenNotFound(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}


#[async_trait::async_trait]
pub trait BannedTokenStore {
    async fn add_token(&mut self, token: &str) -> Result<(), BannedTokenStoreError>;
    async fn is_token_banned(&self, token: &str) -> Result<bool, BannedTokenStoreError>;
    async fn remove_token(&mut self, token: &str) -> Result<(), BannedTokenStoreError>;
}
