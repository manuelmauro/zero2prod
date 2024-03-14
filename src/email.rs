use std::time;

use anyhow::Context;
use reqwest::Client;
use secrecy::{ExposeSecret, Secret};

use crate::domain::subscriber::email::Email;

#[derive(Clone)]
pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: Email,
    authorization_token: Secret<String>,
}

impl EmailClient {
    pub fn new(
        base_url: String,
        sender: Email,
        authorization_token: Secret<String>,
        timeout: time::Duration,
    ) -> Result<Self, anyhow::Error> {
        let http_client = Client::builder().timeout(timeout).build().context("TLS backend cannot be initialized, or the resolver cannot load the system configuration.")?;

        Ok(Self {
            http_client,
            base_url,
            sender,
            authorization_token,
        })
    }

    pub async fn send_email(
        &self,
        recipient: &Email,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/email", self.base_url);
        let request_body = SendEmailRequest {
            from: self.sender.as_ref(),
            to: recipient.as_ref(),
            subject,
            html_body: html_content,
            text_body: text_content,
        };
        self.http_client
            .post(&url)
            .header(
                "X-Postmark-Server-Token",
                self.authorization_token.expose_secret(),
            )
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_body: &'a str,
    text_body: &'a str,
}

#[cfg(test)]
mod tests {
    use std::time;

    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    use secrecy::Secret;
    use wiremock::matchers::{any, header, header_exists, method, path};
    use wiremock::{Mock, MockServer, Request, ResponseTemplate};

    use crate::domain::subscriber::email::Email;
    use crate::email::EmailClient;

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
            } else {
                false
            }
        }
    }

    /// Generate a random email subject
    fn subject() -> String {
        Sentence(1..2).fake()
    }

    /// Generate a random email content
    fn content() -> String {
        Paragraph(1..10).fake()
    }

    /// Generate a random subscriber email
    fn email() -> Email {
        SafeEmail()
            .fake::<String>()
            .try_into()
            .expect("Fake email should be a valid email.")
    }

    /// Get a test instance of `EmailClient`.
    fn email_client(base_url: String) -> Result<EmailClient, anyhow::Error> {
        EmailClient::new(
            base_url,
            email(),
            Secret::new(Faker.fake()),
            time::Duration::from_millis(200),
        )
    }

    #[tokio::test]
    async fn send_email_sends_the_expected_request() {
        let mock_server = MockServer::start().await;
        Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let email_client =
            email_client(mock_server.uri()).expect("The email client should be available");
        let _ = email_client
            .send_email(&email(), &subject(), &content(), &content())
            .await;
    }

    #[tokio::test]
    async fn send_email_succeeds_if_the_server_returns_200() {
        let mock_server = MockServer::start().await;
        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let email_client =
            email_client(mock_server.uri()).expect("The email client should be available");
        let outcome = email_client
            .send_email(&email(), &subject(), &content(), &content())
            .await;

        assert!(outcome.is_ok());
    }

    #[tokio::test]
    async fn send_email_fails_if_the_server_returns_500() {
        let mock_server = MockServer::start().await;
        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        let email_client =
            email_client(mock_server.uri()).expect("The email client should be available");
        let outcome = email_client
            .send_email(&email(), &subject(), &content(), &content())
            .await;

        assert!(outcome.is_err());
    }

    #[tokio::test]
    async fn send_email_times_out_if_the_server_takes_too_long() {
        let mock_server = MockServer::start().await;
        let response = ResponseTemplate::new(200).set_delay(time::Duration::from_secs(180));
        Mock::given(any())
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;

        let email_client =
            email_client(mock_server.uri()).expect("The email client should be available");
        let outcome = email_client
            .send_email(&email(), &subject(), &content(), &content())
            .await;

        assert!(outcome.is_err());
    }
}
