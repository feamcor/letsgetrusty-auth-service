use crate::domain::Email;
use crate::domain::Secret;
use crate::services::EmailClient;
use crate::services::EmailClientError;
use crate::services::EmailClientResult;

const POSTMARK_AUTH_HEADER: &str = "X-Postmark-Server-Token";

pub struct PostmarkEmailClient {
    http_client: reqwest::Client,
    api_key: Secret,
    api_url: url::Url,
    stream: String,
    sender: Email,
}

impl PostmarkEmailClient {
    #[must_use]
    pub fn new(
        http_client: reqwest::Client,
        api_key: Secret,
        api_url: url::Url,
        stream: String,
        sender: Email,
    ) -> Self {
        Self {
            http_client,
            api_key,
            api_url,
            stream,
            sender,
        }
    }
}

// For more information about the request structure, see the API docs: https://postmarkapp.com/developer/user-guide/send-email-with-api
#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct PostmarkSendEmailRequest<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_body: &'a str,
    text_body: &'a str,
    message_stream: &'a str,
}

#[async_trait::async_trait]
impl EmailClient for PostmarkEmailClient {
    #[tracing::instrument(name = "SendingEmail", skip_all)]
    async fn send_email(&self, recipient: &Email, subject: &str, content: &str) -> EmailClientResult<()> {
        let mut url = self.api_url.clone();
        // Ensure the configured base URL is treated as a directory so `join` appends rather than
        // replacing the last path segment (e.g. "/v1" + "email" → "/v1/email", not "/email").
        if !url.path().ends_with('/') {
            let path = format!("{}/", url.path());
            url.set_path(&path);
        }
        let url = url
            .join("email")
            .map_err(|error| EmailClientError::UnexpectedError(error.into()))?;
        let request_body = PostmarkSendEmailRequest {
            from: self.sender.as_secret().expose(),
            to: recipient.as_secret().expose(),
            subject,
            html_body: content,
            text_body: content,
            message_stream: &self.stream,
        };
        self.http_client
            .post(url)
            .header(POSTMARK_AUTH_HEADER, self.api_key.expose())
            .json(&request_body)
            .send()
            .await
            .map_err(|error| EmailClientError::UnexpectedError(error.into()))?
            .error_for_status()
            .map_err(|error| EmailClientError::UnexpectedError(error.into()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::Paragraph;
    use fake::faker::lorem::en::Sentence;
    use fake::Fake;
    use fake::Faker;
    use wiremock::matchers::any;
    use wiremock::matchers::header;
    use wiremock::matchers::header_exists;
    use wiremock::matchers::method;
    use wiremock::matchers::path;
    use wiremock::Mock;
    use wiremock::MockServer;
    use wiremock::Request;
    use wiremock::ResponseTemplate;

    fn subject() -> String {
        Sentence(1..2).fake()
    }

    fn content() -> String {
        Paragraph(1..10).fake()
    }

    fn email() -> Email {
        let fake_email = SafeEmail().fake::<String>().into();
        Email::parse(&fake_email).unwrap()
    }

    fn email_client(base_url: &str) -> PostmarkEmailClient {
        let api_key = Faker.fake::<String>().into();
        let base_url = url::Url::parse(base_url).unwrap();
        let timeout = std::time::Duration::from_millis(200);
        let email = email();
        let stream = String::from("outbound");
        let http_client = reqwest::Client::builder().timeout(timeout).build().unwrap();
        PostmarkEmailClient::new(http_client, api_key, base_url, stream, email)
    }

    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);
            if let Ok(body) = result {
                body.get("From").is_some()
                    && body.get("To").is_some()
                    && body.get("Subject").is_some()
                    && body.get("HtmlBody").is_some()
                    && body.get("TextBody").is_some()
                    && body.get("MessageStream").is_some()
            } else {
                false
            }
        }
    }

    #[tokio::test]
    async fn send_email_sends_the_expected_request() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(&mock_server.uri());
        Mock::given(header_exists(POSTMARK_AUTH_HEADER))
            .and(header(reqwest::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref()))
            .and(path("/email"))
            .and(method(reqwest::Method::POST))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;
        let response = email_client.send_email(&email(), &subject(), &content()).await;
        assert!(response.is_ok());
    }

    #[tokio::test]
    async fn send_email_fails_if_the_server_returns_500() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(&mock_server.uri());
        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;
        let response = email_client.send_email(&email(), &subject(), &content()).await;
        assert!(response.is_err());
    }

    #[tokio::test]
    async fn send_email_times_out_if_the_server_takes_too_long() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(&mock_server.uri());
        let template = ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(180));
        Mock::given(any())
            .respond_with(template)
            .expect(1)
            .mount(&mock_server)
            .await;
        let response = email_client.send_email(&email(), &subject(), &content()).await;
        assert!(response.is_err());
    }
}
