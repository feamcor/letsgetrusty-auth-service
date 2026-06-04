use crate::domain::Email;
use crate::domain::LoginAttemptId;
use crate::domain::TwoFactorAuthCode;

/// Failure modes of a [`TwoFactorAuthCodeStore`] operation.
#[derive(thiserror::Error, Debug)]
pub enum TwoFactorAuthCodeStoreError {
    #[error("Login Attempt ID not found")]
    LoginAttemptIdNotFound,
    #[error("2FA code not found")]
    CodeNotFound,
    #[error(transparent)]
    UnexpectedError(#[from] color_eyre::eyre::Report),
}

/// Convenience alias for a fallible [`TwoFactorAuthCodeStore`] operation.
pub type TwoFactorAuthCodeStoreResult<T> = Result<T, TwoFactorAuthCodeStoreError>;

/// Short-lived storage for pending 2FA codes, keyed by email and expiring on the 2FA TTL.
/// Backed by an in-memory map or Redis.
#[async_trait::async_trait]
pub trait TwoFactorAuthCodeStore: Send + Sync {
    /// Store (or overwrite) the pending code and login-attempt id for an email.
    ///
    /// # Errors
    ///
    /// Returns [`TwoFactorAuthCodeStoreError::UnexpectedError`] on a backend failure.
    async fn add_code(
        &self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFactorAuthCode,
    ) -> TwoFactorAuthCodeStoreResult<()>;

    /// Delete any pending code for an email (idempotent).
    ///
    /// # Errors
    ///
    /// Returns [`TwoFactorAuthCodeStoreError::UnexpectedError`] on a backend failure.
    async fn remove_code(&self, email: &Email) -> TwoFactorAuthCodeStoreResult<()>;

    /// Fetch the pending code and login-attempt id for an email.
    ///
    /// # Errors
    ///
    /// Returns [`TwoFactorAuthCodeStoreError::CodeNotFound`] if none is pending or it has expired.
    async fn get_code(&self, email: &Email) -> TwoFactorAuthCodeStoreResult<(LoginAttemptId, TwoFactorAuthCode)>;
}

crate::services::arc_dyn::arc_dyn_newtype! {
    /// Shared, cloneable handle to the active [`TwoFactorAuthCodeStore`] implementation.
    TwoFactorAuthCodeStoreType, TwoFactorAuthCodeStore
}
