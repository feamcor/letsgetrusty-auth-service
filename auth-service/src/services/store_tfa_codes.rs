use crate::domain::Email;
use crate::domain::LoginAttemptId;
use crate::domain::TwoFactorAuthCode;

#[derive(thiserror::Error, Debug)]
pub enum TwoFactorAuthCodeStoreError {
    #[error("Login Attempt ID not found")]
    LoginAttemptIdNotFound,
    #[error("2FA code not found")]
    CodeNotFound,
    #[error(transparent)]
    UnexpectedError(#[from] color_eyre::eyre::Report),
}

pub type TwoFactorAuthCodeStoreResult<T> = Result<T, TwoFactorAuthCodeStoreError>;

#[async_trait::async_trait]
pub trait TwoFactorAuthCodeStore: Send + Sync {
    async fn add_code(
        &self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFactorAuthCode,
    ) -> TwoFactorAuthCodeStoreResult<()>;
    async fn remove_code(&self, email: &Email) -> TwoFactorAuthCodeStoreResult<()>;
    async fn get_code(&self, email: &Email) -> TwoFactorAuthCodeStoreResult<(LoginAttemptId, TwoFactorAuthCode)>;
}

#[derive(Clone)]
pub struct TwoFactorAuthCodeStoreType {
    inner: std::sync::Arc<dyn TwoFactorAuthCodeStore>,
}

impl TwoFactorAuthCodeStoreType {
    pub fn new(inner: impl TwoFactorAuthCodeStore + 'static) -> Self {
        Self {
            inner: std::sync::Arc::new(inner),
        }
    }

    #[must_use]
    pub fn inner(&self) -> std::sync::Arc<dyn TwoFactorAuthCodeStore> {
        self.inner.clone()
    }
}

impl std::fmt::Debug for TwoFactorAuthCodeStoreType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TwoFactorAuthCodeStoreType").finish_non_exhaustive()
    }
}
