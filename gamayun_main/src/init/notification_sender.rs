use crate::config::app_config::AppConfig;
use crate::notification::composite_notification_sender::CompositeNotificationSender;
use crate::notification::mail::sendgrid_notification_sender::{
    SendGridConfiguration, SendGridNotificationSender,
};
use crate::notification::NotificationSender;
use std::sync::Arc;

/// Initializes a `CompositeNotificationSender` with available notification senders.
///
/// This function takes the application's configuration and initializes all configured
/// notification senders, such as SendGrid. It aggregates these senders into a
/// `CompositeNotificationSender`, which can then be used to send notifications through
/// multiple channels.
///
/// # Parameters
///
/// - `app_config`: The application's configuration containing settings for various
///   notification senders.
///
/// # Returns
///
/// A `CompositeNotificationSender` instance containing all successfully initialized
/// notification senders.
pub fn initialize_notification_sender(app_config: AppConfig) -> CompositeNotificationSender {
    let senders_opt = vec![initialize_send_grid_notifier(app_config)];

    let senders: Vec<Arc<dyn NotificationSender>> = senders_opt
        .into_iter()
        .filter_map(|sender| sender.map(|s| Arc::new(s) as Arc<dyn NotificationSender>))
        .collect();

    let composite_sender = CompositeNotificationSender::new(Some(senders));
    composite_sender
}

/// Initializes a `SendGridNotificationSender` if SendGrid is configured.
///
/// This function checks the application configuration for SendGrid settings. If
/// SendGrid is configured, it creates a new `SendGridNotificationSender` with the
/// provided settings.
///
/// # Parameters
///
/// - `app_config`: The application's configuration containing SendGrid settings.
///
/// # Returns
///
/// An `Option<SendGridNotificationSender>`. Returns `Some` if SendGrid is configured;
/// otherwise, returns `None`.
fn initialize_send_grid_notifier(app_config: AppConfig) -> Option<SendGridNotificationSender> {
    app_config.sendgrid_config.map(|sendgrid_config| {
        SendGridNotificationSender::new(SendGridConfiguration {
            from_email: sendgrid_config.from_email,
            to_emails: sendgrid_config.to_emails,
            api_key: sendgrid_config.api_key,
        })
    })
}
