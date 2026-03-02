use crate::domain::{Email, LoginAttemptId, TwoFactorAuthCode};

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
pub trait TwoFactorAuthCodeStore {
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
