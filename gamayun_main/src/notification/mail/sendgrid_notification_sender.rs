use crate::notification::NotificationSender;
use async_trait::async_trait;
use reqwest::Client;
use std::env;
use tracing::instrument;

/// Configuration struct for SendGrid.
pub struct SendGridConfiguration {
    pub from_email: String,
    pub to_emails: Vec<String>,
}

/// Implementation of `NotificationSender` for SendGrid.
pub struct SendGridNotificationSender {
    config: SendGridConfiguration,
    client: Client,
    api_key: String,
}

impl SendGridNotificationSender {
    /// Creates a new `SendGridNotificationSender` with a configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - A `SendGridConfiguration` containing the "from" email and recipient emails.
    pub fn new(config: SendGridConfiguration) -> Self {
        let api_key =
            env::var("GAMAYUN_SENDGRID_API_KEY").expect("GAMAYUN_SENDGRID_API_KEY not set");
        SendGridNotificationSender {
            config,
            client: Client::new(),
            api_key,
        }
    }
}

#[async_trait]
impl NotificationSender for SendGridNotificationSender {
    #[instrument(skip(self, message_contents))]
    async fn notify(&self, message_title: String, message_contents: String) {
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
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await;

        if let Err(e) = response {
            eprintln!("Failed to send notification via SendGrid: {:?}", e);
        }
    }
}
