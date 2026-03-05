use crate::domain::Email;
use std::sync::Arc;

#[derive(thiserror::Error, Debug)]
pub enum EmailClientError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

#[async_trait::async_trait]
pub trait EmailClient: Send + Sync {
    async fn send_email(
        &self,
        recipient: &Email,
        subject: &str,
        content: &str,
    ) -> Result<(), EmailClientError>;
}

#[derive(Clone)]
pub struct EmailClientType {
    inner: Arc<dyn EmailClient>,
}

impl EmailClientType {
    pub fn new(inner: impl EmailClient + 'static) -> Self {
        Self {
            inner: Arc::new(inner),
        }
    }

    pub fn inner(&self) -> Arc<dyn EmailClient> {
        self.inner.clone()
    }
}

impl std::fmt::Debug for EmailClientType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EmailClientType").finish_non_exhaustive()
    }
}
