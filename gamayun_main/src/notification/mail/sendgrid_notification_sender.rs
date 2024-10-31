use crate::notification::NotificationSender;
use async_trait::async_trait;
use reqwest::Client;
use tracing::{error, info, instrument};

/// Configuration struct for SendGrid.
pub struct SendGridConfiguration {
    pub from_email: String,
    pub to_emails: Vec<String>,
    pub api_key: String,
}

/// Implementation of `NotificationSender` for SendGrid.
pub struct SendGridNotificationSender {
    config: SendGridConfiguration,
    client: Client,
}

impl SendGridNotificationSender {
    /// Creates a new `SendGridNotificationSender` with a configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - A `SendGridConfiguration` containing the "from" email and recipient emails.
    pub fn new(config: SendGridConfiguration) -> Self {
        SendGridNotificationSender {
            config,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl NotificationSender for SendGridNotificationSender {
    #[instrument(skip(self, message_contents))]
    async fn notify(&self, message_title: String, message_contents: String) {
        info!(
            "Sending notification via SendGrid with title {}",
            &message_title
        );
        let url = "https://api.sendgrid.com/v3/mail/send";

        let body = serde_json::json!({
            "personalizations": [{
                "to": self.config.to_emails.iter().map(|email| serde_json::json!({ "email": email })).collect::<Vec<_>>(),
                "subject": message_title,
            }],
            "from": { "email": self.config.from_email },
            "content": [{
                "type": "text/plain",
                "value": message_contents,
            }],
        });

        let response = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await;

        if let Err(e) = response {
            error!("Failed to send notification via SendGrid: {:?}", e);
        }
    }
}
