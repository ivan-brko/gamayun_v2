mod composite_notification_sender;
mod mail;

use async_trait::async_trait;

/// Defines a trait for sending notifications asynchronously.
#[async_trait]
pub trait NotificationSender: Send + Sync {
    /// Sends a notification with the given title and contents.
    ///
    /// # Arguments
    ///
    /// * `message_title` - A `String` containing the notification title.
    /// * `message_contents` - A `String` containing the notification message.
    async fn notify(&self, message_title: String, message_contents: String);
}
