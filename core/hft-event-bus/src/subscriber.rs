//! Subscriber utilities and helpers

use crate::events::{Event, EventEnvelope};
use tokio::sync::broadcast;

/// Helper for subscribing to specific event types
pub struct Subscriber {
    receiver: broadcast::Receiver<EventEnvelope>,
}

impl Subscriber {
    /// Create from broadcast receiver
    pub fn new(receiver: broadcast::Receiver<EventEnvelope>) -> Self {
        Self { receiver }
    }
    
    /// Receive next event
    pub async fn recv(&mut self) -> Option<EventEnvelope> {
        match self.receiver.recv().await {
            Ok(event) => Some(event),
            Err(broadcast::error::RecvError::Lagged(skipped)) => {
                tracing::warn!("Subscriber lagged, skipped {} events", skipped);
                None
            }
            Err(broadcast::error::RecvError::Closed) => None,
        }
    }
    
    /// Try to receive without blocking
    pub fn try_recv(&mut self) -> Option<EventEnvelope> {
        match self.receiver.try_recv() {
            Ok(event) => Some(event),
            Err(_) => None,
        }
    }
    
    /// Resubscribe (useful after lagging)
    pub fn resubscribe(&self) -> Self {
        Self {
            receiver: self.receiver.resubscribe(),
        }
    }
}
