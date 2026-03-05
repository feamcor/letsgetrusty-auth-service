use std::sync::Arc;

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
pub trait BannedTokenStore: Send + Sync {
    async fn add_token(&self, token: &str) -> Result<(), BannedTokenStoreError>;
    async fn is_token_banned(&self, token: &str) -> Result<bool, BannedTokenStoreError>;
    async fn remove_token(&self, token: &str) -> Result<(), BannedTokenStoreError>;
}

#[derive(Clone)]
pub struct BannedTokenStoreType {
    inner: Arc<dyn BannedTokenStore>,
}

impl BannedTokenStoreType {
    pub fn new(inner: impl BannedTokenStore + 'static) -> Self {
        Self {
            inner: Arc::new(inner),
        }
    }

    pub fn inner(&self) -> Arc<dyn BannedTokenStore> {
        self.inner.clone()
    }
}

impl std::fmt::Debug for BannedTokenStoreType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BannedTokenStoreType")
            .finish_non_exhaustive()
    }
}
