use crate::domain::subscriber_name::SubscriberName;

/// Captures all of the information we need to register a new subscriber.
pub struct NewSubscriber {
    pub email: String,
    pub name: SubscriberName,
}
