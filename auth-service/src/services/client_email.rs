use crate::domain::Email;

#[derive(thiserror::Error, Debug)]
pub enum EmailClientError {
    #[error(transparent)]
    UnexpectedError(#[from] color_eyre::eyre::Report),
}

pub type EmailClientResult<T> = Result<T, EmailClientError>;

#[async_trait::async_trait]
pub trait EmailClient: Send + Sync {
    async fn send_email(&self, recipient: &Email, subject: &str, content: &str) -> EmailClientResult<()>;
}

#[derive(Clone)]
pub struct EmailClientType {
    inner: std::sync::Arc<dyn EmailClient>,
}

impl EmailClientType {
    pub fn new(inner: impl EmailClient + 'static) -> Self {
        Self {
            inner: std::sync::Arc::new(inner),
        }
    }

    #[must_use]
    pub fn inner(&self) -> std::sync::Arc<dyn EmailClient> {
        self.inner.clone()
    }
}

impl std::fmt::Debug for EmailClientType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EmailClientType").finish_non_exhaustive()
    }
}
