#[derive(thiserror::Error, Debug)]
pub enum BannedTokenStoreError {
    #[error("Token already exists: {0}")]
    TokenAlreadyExists(String),
    #[error("Token was not found: {0}")]
    TokenNotFound(String),
    #[error(transparent)]
    UnexpectedError(#[from] color_eyre::eyre::Report),
}

pub type BannedTokenStoreResult<T> = Result<T, BannedTokenStoreError>;

#[async_trait::async_trait]
pub trait BannedTokenStore: Send + Sync {
    async fn add_token(&self, token: &str) -> BannedTokenStoreResult<()>;
    async fn is_token_banned(&self, token: &str) -> BannedTokenStoreResult<bool>;
    async fn remove_token(&self, token: &str) -> BannedTokenStoreResult<()>;
}

#[derive(Clone)]
pub struct BannedTokenStoreType {
    inner: std::sync::Arc<dyn BannedTokenStore>,
}

impl BannedTokenStoreType {
    pub fn new(inner: impl BannedTokenStore + 'static) -> Self {
        Self {
            inner: std::sync::Arc::new(inner),
        }
    }

    #[must_use]
    pub fn inner(&self) -> std::sync::Arc<dyn BannedTokenStore> {
        self.inner.clone()
    }
}

impl std::fmt::Debug for BannedTokenStoreType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BannedTokenStoreType").finish_non_exhaustive()
    }
}
