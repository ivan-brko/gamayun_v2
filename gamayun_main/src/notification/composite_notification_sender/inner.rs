use crate::notification::NotificationSender;
use std::sync::Arc; // Keep this for shared ownership of senders.
use tokio::sync::Mutex;
use tracing::error;
// Use tokio's async Mutex for async scenarios.

/// Internal struct that holds and manages the list of notification senders.
pub struct CompositeNotificationSenderInner {
    senders: Mutex<Vec<Arc<dyn NotificationSender>>>,
}

impl CompositeNotificationSenderInner {
    /// Creates a new `CompositeNotificationSenderInner` with an initial list of senders.
    ///
    /// # Arguments
    ///
    /// * `initial_senders` - A vector of objects implementing the `NotificationSender` trait.
    pub fn new(initial_senders: Vec<Arc<dyn NotificationSender>>) -> Self {
        CompositeNotificationSenderInner {
            senders: Mutex::new(initial_senders),
        }
    }

    /// Adds a new `NotificationSender` to the composite.
    ///
    /// # Arguments
    ///
    /// * `sender` - An `Arc` pointing to an object that implements the `NotificationSender` trait.
    pub async fn add_sender(&self, sender: Arc<dyn NotificationSender>) {
        let mut senders = self.senders.lock().await;
        senders.push(sender);
    }

    /// Notifies all senders with the given title and contents.
    ///
    /// # Arguments
    ///
    /// * `message_title` - A shared reference to the notification title.
    /// * `message_contents` - A shared reference to the notification message.
    pub async fn notify(&self, message_title: String, message_contents: String) {
        let senders = self.senders.lock().await;
        let mut handles = Vec::new();
        for sender in senders.iter() {
            let sender_clone = Arc::clone(sender);
            let title_clone = message_title.clone();
            let contents_clone = message_contents.clone();
            let handle = tokio::spawn(async move {
                sender_clone.notify(title_clone, contents_clone).await;
            });
            handles.push(handle);
        }

        for handle in handles {
            if let Err(e) = handle.await {
                error!("Notification task failed: {:?}", e);
            }
        }
    }
}
