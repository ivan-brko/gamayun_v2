use crate::notification::composite_notification_sender::CompositeNotificationSender;

pub fn initialize_notification_sender() -> CompositeNotificationSender {
    // let mail_sender = MailNotificationSender::new();
    // let composite_sender = CompositeNotificationSender::new(Some(vec![Arc::new(mail_sender)]));
    let composite_sender = CompositeNotificationSender::new(Some(vec![]));
    composite_sender
}
