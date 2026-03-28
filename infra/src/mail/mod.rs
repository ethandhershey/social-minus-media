use domain::{error::DomainError, ports::MailService};
use resend_rs::{Resend, types::CreateEmailBaseOptions};
use secrecy::{ExposeSecret, SecretString};

use crate::http::HttpClient;

pub struct ResendMailClient {
    client: Resend,
    sender_email: String,
    sender_name: String,
}

impl ResendMailClient {
    pub fn new(
        client: HttpClient,
        api_key: SecretString,
        sender_email: String,
        sender_name: String,
    ) -> Self {
        Self {
            client: Resend::with_client(api_key.expose_secret(), client.into()),
            sender_email,
            sender_name,
        }
    }
}

impl MailService for ResendMailClient {
    async fn send(
        &self,
        to_email: &str,
        to_name: &str,
        subject: &str,
        html_body: &str,
    ) -> Result<(), DomainError> {
        let from = format!("{} <{}>", self.sender_name, self.sender_email);
        let to = [format!("{to_name} <{to_email}>")];

        let email = CreateEmailBaseOptions::new(from, to, subject).with_html(html_body);

        self.client.emails.send(email).await.map_err(|e| {
            tracing::error!("Resend request failed: {e}");
            DomainError::MailServiceUnavailable
        })?;

        Ok(())
    }
}
