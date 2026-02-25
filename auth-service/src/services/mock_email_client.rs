use tracing::info;
use crate::domain::Email;
use crate::services::{EmailClient, EmailClientError};

#[derive(Debug, Clone)]
pub struct MockEmailClient;

#[async_trait::async_trait]
impl EmailClient for MockEmailClient {
    async fn send_email(
        &self,
        recipient: &Email,
        subject: &str,
        content: &str,
    ) -> Result<(), EmailClientError> {
        info!(
            "sent email to={} subject='{}' content='{}'",
            recipient.as_ref(),
            subject,
            content
        );
        Ok(())
    }
}
