use std::time::Duration;
use reqwest::Client;
use secrecy::{ExposeSecret, Secret};
use crate::domain::AccountEmail;

pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: AccountEmail,
    auth_token: Secret<String>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest<'s> {
    from: &'s str,
    to: &'s str,
    subject: &'s str,
    html_message: &'s str,
    plain_message: &'s str,
}

impl EmailClient {
    pub fn new(
        base_url: String,
        sender: AccountEmail,
        auth_token: Secret<String>,
        timeout: Duration,
    ) -> Self {
        let http_client = Client::builder().timeout(timeout).build().unwrap();
        EmailClient {
            http_client,
            base_url,
            sender,
            auth_token,
        }
    }

    pub async fn send_email(
        &self,
        recipient: &AccountEmail,
        subject: &str,
        html_message: &str,
        plain_message: &str,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/email", self.base_url);

        let request_body = SendEmailRequest {
            from: self.sender.as_ref(),
            to: recipient.as_ref(),
            subject,
            html_message,
            plain_message,
        };

        let _builder = self.http_client
            .post(&url)
            .header(
                "X-Authorization-Token",
                self.auth_token.expose_secret(),
            )
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use claim::assert_err;
    use fake::{Fake, Faker};
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use secrecy::Secret;
    use wiremock::{Match, Mock, MockServer, Request, ResponseTemplate};
    use wiremock::matchers::{any, header, header_exists, method, path};
    use crate::domain::AccountEmail;
    use crate::email_client::EmailClient;

    struct EmailBodyMatcher;

    impl Match for EmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);
            if let Ok(body) = result {
                body.get("From").is_some()
                    && body.get("To").is_some()
                    && body.get("Subject").is_some()
                    && body.get("HtmlMessage").is_some()
                    && body.get("PlainMessage").is_some()
            } else {
                false
            }
        }
    }

    async fn create_mock_get_client() -> (MockServer, EmailClient) {
        let server = MockServer::start().await;
        let client = email_client(server.uri());
        (server, client)
    }

    fn subject() -> String {
        Sentence(1..4).fake()
    }

    fn content() -> String {
        Paragraph(2..10).fake()
    }

    fn email() -> AccountEmail {
        AccountEmail::parse(SafeEmail().fake()).unwrap()
    }

    fn email_client(base_url: String) -> EmailClient {
        EmailClient::new(
            base_url,
            email(),
            Secret::new(Faker.fake()),
            Duration::from_millis(200),
        )
    }

    #[tokio::test]
    async fn send_email_succeeds_if_the_server_returns_200() {
        let (server, client) = create_mock_get_client().await;
        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;

        let _ = client
            .send_email(&email(), &subject(), &content(), &content())
            .await;
    }

    #[tokio::test]
    async fn send_email_times_out_if_the_server_takes_too_long() {
        let (server, client) = create_mock_get_client().await;
        let response = ResponseTemplate::new(200)
            .set_delay(Duration::from_secs(180));

        Mock::given(any())
            .respond_with(response)
            .expect(1)
            .mount(&server)
            .await;

        let outcome = client
            .send_email(&email(), &subject(), &content(), &content())
            .await;

        assert_err!(outcome);
    }

    #[tokio::test]
    async fn send_email_fails_if_server_returns_500() {
        let (server, client) = create_mock_get_client().await;
        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&server)
            .await;

        let outcome = client
            .send_email(&email(), &subject(), &content(), &content())
            .await;

        assert_err!(outcome);
    }

    #[tokio::test]
    async fn email_is_accepted_if_header_is_present() {
        let (server, client) = create_mock_get_client().await;
        Mock::given(header_exists("X-Authorization-Token"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;

        let _ = client
            .send_email(&email(), &subject(), &content(), &content())
            .await;
    }

    #[tokio::test]
    async fn send_email_sends_expected_request() {
        let (server, client) = create_mock_get_client().await;
        Mock::given(header_exists("X-Authorization-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .and(EmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;

        let _ = client
            .send_email(&email(), &subject(), &content(), &content())
            .await;
    }
}