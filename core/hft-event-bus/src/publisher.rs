//! Publisher utilities and helpers

use crate::bus::EventBus;
use crate::events::Event;
use anyhow::Result;
use std::sync::Arc;

/// Helper for publishing events
pub struct Publisher {
    bus: Arc<EventBus>,
}

impl Publisher {
    /// Create new publisher
    pub fn new(bus: Arc<EventBus>) -> Self {
        Self { bus }
    }
    
    /// Publish event with default priority
    pub async fn publish<T: Event + Send + 'static>(&self, event: T) -> Result<()> {
        self.bus.publish(event).await
    }
    
    /// Publish event with high priority
    pub async fn publish_high_priority<T: Event + Send + 'static>(&self, event: T) -> Result<()> {
        self.bus.publish_with_priority(event, 0).await
    }
    
    /// Publish event with low priority
    pub async fn publish_low_priority<T: Event + Send + 'static>(&self, event: T) -> Result<()> {
        self.bus.publish_with_priority(event, 9).await
    }
}
