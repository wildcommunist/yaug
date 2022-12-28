use std::time::Duration;
use reqwest::Client;
use secrecy::Secret;
use crate::domain::UserEmail;

pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: UserEmail,
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
        sender: UserEmail,
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
}