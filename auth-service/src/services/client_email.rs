use crate::domain::Email;

/// Failure modes of an [`EmailClient`] operation.
#[derive(thiserror::Error, Debug)]
pub enum EmailClientError {
    #[error(transparent)]
    UnexpectedError(#[from] color_eyre::eyre::Report),
}

/// Convenience alias for a fallible [`EmailClient`] operation.
pub type EmailClientResult<T> = Result<T, EmailClientError>;

/// Outbound transactional email (e.g. 2FA codes), backed by a no-op mock or the Postmark API.
#[async_trait::async_trait]
pub trait EmailClient: Send + Sync {
    /// Send a plain-text email to a single recipient.
    ///
    /// # Errors
    ///
    /// Returns [`EmailClientError::UnexpectedError`] if the message could not be dispatched.
    async fn send_email(&self, recipient: &Email, subject: &str, content: &str) -> EmailClientResult<()>;
}

crate::services::arc_dyn::arc_dyn_newtype! {
    /// Shared, cloneable handle to the active [`EmailClient`] implementation.
    EmailClientType, EmailClient
}
