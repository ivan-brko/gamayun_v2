use crate::notification::composite_notification_sender::inner::CompositeNotificationSenderInner;
use crate::notification::NotificationSender;
use async_trait::async_trait;
use std::sync::Arc;

mod inner;

/// A composite notification sender that holds multiple senders and
/// dispatches notifications to all of them.
/// It's thread-safe, it can be cloned
#[derive(Clone)]
pub struct CompositeNotificationSender {
    inner: Arc<CompositeNotificationSenderInner>,
}
impl CompositeNotificationSender {
    /// Creates a new `composite_notification_sender` with an optional initial list of senders.
    ///
    /// # Arguments
    ///
    /// * `initial_senders` - An optional vector of `Arc` objects implementing the `NotificationSender` trait.
    ///
    /// # Example
    ///
    /// ```rust
    /// let composite_sender = composite_notification_sender::new(Some(vec![sender1, sender2]));
    /// ```
    pub fn new(initial_senders: Option<Vec<Arc<dyn NotificationSender>>>) -> Self {
        let senders = initial_senders.unwrap_or_else(Vec::new);
        CompositeNotificationSender {
            inner: Arc::new(CompositeNotificationSenderInner::new(senders)),
        }
    }

    /// Adds a new `NotificationSender` to the composite.
    ///
    /// # Arguments
    ///
    /// * `sender` - An object that implements the `NotificationSender` trait.
    ///
    /// # Example
    ///
    /// ```rust
    /// composite_sender.add_sender(new_sender).await;
    /// ```
    pub async fn add_sender<S>(&self, sender: S)
    where
        S: NotificationSender + 'static,
    {
        self.inner.add_sender(Arc::new(sender)).await;
    }
}

#[async_trait]
impl NotificationSender for CompositeNotificationSender {
    async fn notify(&self, message_title: String, message_contents: String) {
        self.inner.notify(message_title, message_contents).await;
    }
}
