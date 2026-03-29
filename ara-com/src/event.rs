use crate::types::*;
use crate::error::AraComError;
use futures_core::Stream;
use std::pin::Pin;

/// Subscription state for event consumers
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubscriptionState {
    /// Not subscribed
    Idle,
    /// Subscription requested, waiting for ack
    Pending,
    /// Actively subscribed
    Subscribed,
    /// Subscription was rejected or lost
    Failed(String),
}

/// Configuration for event subscriptions
#[derive(Debug, Clone)]
pub struct EventConfig {
    pub event_group_id: EventGroupId,
    /// Max number of buffered events before dropping
    pub buffer_size: usize,
}

impl EventConfig {
    /// Create a new `EventConfig` with sensible defaults (buffer size = 64).
    pub fn new(event_group_id: EventGroupId) -> Self {
        Self {
            event_group_id,
            buffer_size: 64,
        }
    }
}

/// A stream of events from a subscribed event group
pub type EventStream<T> = Pin<Box<dyn Stream<Item = Result<T, AraComError>> + Send>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_config_new() {
        let cfg = EventConfig::new(EventGroupId(0x0010));
        assert_eq!(cfg.event_group_id, EventGroupId(0x0010));
        assert_eq!(cfg.buffer_size, 64);
    }

    #[test]
    fn test_subscription_state_equality() {
        assert_eq!(SubscriptionState::Idle, SubscriptionState::Idle);
        assert_ne!(SubscriptionState::Idle, SubscriptionState::Subscribed);
        assert_eq!(
            SubscriptionState::Failed("oops".into()),
            SubscriptionState::Failed("oops".into())
        );
    }
}
