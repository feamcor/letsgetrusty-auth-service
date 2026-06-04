use crate::domain::Email;
use crate::services::EmailClient;
use crate::services::EmailClientResult;

/// [`EmailClient`] that logs instead of sending — used in development and tests.
#[derive(Debug, Clone)]
pub struct MockEmailClient;

#[async_trait::async_trait]
impl EmailClient for MockEmailClient {
    async fn send_email(&self, _recipient: &Email, subject: &str, _content: &str) -> EmailClientResult<()> {
        tracing::info!(subject = subject, "mock email sent");
        Ok(())
    }
}
