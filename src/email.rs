use reqwest::Client;

use crate::domain::subscriber::email::Email;

#[derive(Clone)]
pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: Email,
}

impl EmailClient {
    pub fn new(base_url: String, sender: Email) -> Self {
        Self {
            http_client: Client::new(),
            base_url,
            sender,
        }
    }

    pub async fn send_email(
        &self,
        recipient: Email,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), String> {
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use crate::email::EmailClient;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::Fake;
    use wiremock::matchers::any;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        // Arrange
        let mock_server = MockServer::start().await;
        let sender = SafeEmail().fake::<String>().try_into().unwrap();
        let email_client = EmailClient::new(mock_server.uri(), sender);

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let subscriber_email = SafeEmail().fake::<String>().try_into().unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        // Act
        let _ = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;

        // Assert
    }
}
