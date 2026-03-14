use crate::domain::Email;
use crate::services::EmailClient;
use crate::services::EmailClientResult;

#[derive(Debug, Clone)]
pub struct MockEmailClient;

#[async_trait::async_trait]
impl EmailClient for MockEmailClient {
    async fn send_email(&self, recipient: &Email, subject: &str, content: &str) -> EmailClientResult<()> {
        tracing::info!(
            "sent email to={} subject='{}' content='{}'",
            recipient.as_secret().expose(),
            subject,
            content
        );
        Ok(())
    }
}
