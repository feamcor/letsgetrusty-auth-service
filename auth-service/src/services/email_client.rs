use crate::domain::Email;

#[derive(thiserror::Error, Debug)]
pub enum EmailClientError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

#[async_trait::async_trait]
pub trait EmailClient {
    async fn send_email(
        &self,
        recipient: &Email,
        subject: &str,
        content: &str,
    ) -> Result<(), EmailClientError>;
}
