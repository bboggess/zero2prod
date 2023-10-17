use super::{SubscriberEmail, SubscriberName};

/// Captures all of the information we need to register a new subscriber.
pub struct NewSubscriber {
    pub email: SubscriberEmail,
    pub name: SubscriberName,
}
