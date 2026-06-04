use crate::domain::Token;

/// Failure modes of a [`BannedTokenStore`] operation.
#[derive(thiserror::Error, Debug)]
pub enum BannedTokenStoreError {
    #[error("Token already exists")]
    TokenAlreadyExists,
    #[error("Token was not found")]
    TokenNotFound,
    #[error(transparent)]
    UnexpectedError(#[from] color_eyre::eyre::Report),
}

/// Convenience alias for a fallible [`BannedTokenStore`] operation.
pub type BannedTokenStoreResult<T> = Result<T, BannedTokenStoreError>;

/// A revocation list of JWTs that must be rejected before their natural expiry (e.g. after
/// logout). Entries expire on the JWT TTL, backed by an in-memory map or Redis.
#[async_trait::async_trait]
pub trait BannedTokenStore: Send + Sync {
    /// Ban a token until it would naturally expire.
    ///
    /// # Errors
    ///
    /// Returns [`BannedTokenStoreError::TokenAlreadyExists`] if the token is already banned.
    async fn add_token(&self, token: &Token) -> BannedTokenStoreResult<()>;

    /// Report whether a token is currently banned (expired entries count as not banned).
    ///
    /// # Errors
    ///
    /// Returns [`BannedTokenStoreError::UnexpectedError`] on a backend failure.
    async fn is_token_banned(&self, token: &Token) -> BannedTokenStoreResult<bool>;

    /// Remove a token from the ban list.
    ///
    /// # Errors
    ///
    /// Returns [`BannedTokenStoreError::TokenNotFound`] if the token was not banned.
    async fn remove_token(&self, token: &Token) -> BannedTokenStoreResult<()>;
}

crate::services::arc_dyn::arc_dyn_newtype! {
    /// Shared, cloneable handle to the active [`BannedTokenStore`] implementation.
    BannedTokenStoreType, BannedTokenStore
}
