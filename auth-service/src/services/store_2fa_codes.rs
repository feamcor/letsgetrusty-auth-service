use crate::domain::{Email, LoginAttemptId, TwoFactorAuthCode};
use std::sync::Arc;

#[derive(thiserror::Error, Debug)]
pub enum TwoFactorAuthCodeStoreError {
    #[error("Login Attempt ID not found")]
    LoginAttemptIdNotFound,
    #[error("2FA code not found")]
    CodeNotFound,
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

#[async_trait::async_trait]
pub trait TwoFactorAuthCodeStore: Send + Sync {
    async fn add_code(
        &self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFactorAuthCode,
    ) -> Result<(), TwoFactorAuthCodeStoreError>;
    async fn remove_code(&self, email: &Email) -> Result<(), TwoFactorAuthCodeStoreError>;
    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFactorAuthCode), TwoFactorAuthCodeStoreError>;
}

#[derive(Clone)]
pub struct TwoFactorAuthCodeStoreType {
    inner: Arc<dyn TwoFactorAuthCodeStore>,
}

impl TwoFactorAuthCodeStoreType {
    pub fn new(inner: impl TwoFactorAuthCodeStore + 'static) -> Self {
        Self {
            inner: Arc::new(inner),
        }
    }

    #[must_use]
    pub fn inner(&self) -> Arc<dyn TwoFactorAuthCodeStore> {
        self.inner.clone()
    }
}

impl std::fmt::Debug for TwoFactorAuthCodeStoreType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TwoFactorAuthCodeStoreType")
            .finish_non_exhaustive()
    }
}
