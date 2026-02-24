//! Core event bus implementation

use crate::events::{Event, EventEnvelope};
use anyhow::Result;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, warn};

/// Channel capacity for each event type
const CHANNEL_CAPACITY: usize = 10000;

/// High-performance event bus for multi-threaded pub/sub
pub struct EventBus {
    /// Broadcast channels for each event type
    channels: Arc<DashMap<String, broadcast::Sender<EventEnvelope>>>,
    
    /// Event recorder for replay (optional)
    recorder: Option<Arc<crate::replay::EventRecorder>>,
    
    /// Statistics
    stats: Arc<DashMap<String, EventStats>>,
}

#[derive(Debug, Clone, Default)]
pub struct EventStats {
    pub published: u64,
    pub received: u64,
    pub dropped: u64,
}

impl EventBus {
    /// Create a new event bus
    pub fn new() -> Self {
        Self {
            channels: Arc::new(DashMap::new()),
            recorder: None,
            stats: Arc::new(DashMap::new()),
        }
    }
    
    /// Create event bus with recording enabled
    pub fn with_recording(capacity: usize) -> Self {
        Self {
            channels: Arc::new(DashMap::new()),
            recorder: Some(Arc::new(crate::replay::EventRecorder::new(capacity))),
            stats: Arc::new(DashMap::new()),
        }
    }
    
    /// Publish an event to all subscribers
    pub async fn publish<T: Event + Send + 'static>(&self, event: T) -> Result<()> {
        self.publish_with_priority(event, 5).await
    }
    
    /// Publish event with specific priority (0 = highest)
    pub async fn publish_with_priority<T: Event + Send + 'static>(&self, event: T, priority: u8) -> Result<()> {
        let event_type = Self::event_type_name(&event);
        let envelope = EventEnvelope::new(event, priority);
        
        // Record event if recording is enabled
        if let Some(recorder) = &self.recorder {
            recorder.record(envelope).await;
        }
        
        // Get or create channel for this event type
        let sender = self.channels.entry(event_type.to_string())
            .or_insert_with(|| {
                debug!("Creating new channel for event type: {}", event_type);
                broadcast::channel(CHANNEL_CAPACITY).0
            })
            .clone();
        
        // Publish to channel
        match sender.send(envelope) {
            Ok(_subscriber_count) => {
                self.increment_stat(event_type, |s| s.published += 1);
                Ok(())
            }
            Err(_) => {
                self.increment_stat(event_type, |s| s.dropped += 1);
                Ok(()) // Not an error if no subscribers
            }
        }
    }
    
    /// Subscribe to a specific event type
    pub async fn subscribe(&self, event_type: &str) -> broadcast::Receiver<EventEnvelope> {
        let sender = self.channels.entry(event_type.to_string())
            .or_insert_with(|| {
                debug!("Creating new channel for subscription: {}", event_type);
                broadcast::channel(CHANNEL_CAPACITY).0
            })
            .clone();
        
        sender.subscribe()
    }
    
    /// Subscribe to market data events
    pub async fn subscribe_market_data(&self) -> broadcast::Receiver<EventEnvelope> {
        self.subscribe("MarketData").await
    }
    
    /// Subscribe to signal events
    pub async fn subscribe_signals(&self) -> broadcast::Receiver<EventEnvelope> {
        self.subscribe("Signal").await
    }
    
    /// Subscribe to fill events
    pub async fn subscribe_fills(&self) -> broadcast::Receiver<EventEnvelope> {
        self.subscribe("Fill").await
    }
    
    /// Subscribe to order events
    pub async fn subscribe_orders(&self) -> broadcast::Receiver<EventEnvelope> {
        self.subscribe("Order").await
    }
    
    /// Subscribe to feature events
    pub async fn subscribe_features(&self) -> broadcast::Receiver<EventEnvelope> {
        self.subscribe("Feature").await
    }
    
    /// Get event statistics
    pub fn get_stats(&self) -> Vec<(String, EventStats)> {
        self.stats.iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }
    
    /// Get event recorder for replay
    pub fn recorder(&self) -> Option<Arc<crate::replay::EventRecorder>> {
        self.recorder.clone()
    }
    
    /// Helper to get event type name (zero-alloc)
    fn event_type_name(event: &dyn Event) -> &'static str {
        event.event_type()
    }
    
    /// Increment statistics
    fn increment_stat<F>(&self, event_type: &str, f: F)
    where
        F: FnOnce(&mut EventStats),
    {
        self.stats.entry(event_type.to_string())
            .or_insert_with(EventStats::default)
            .value_mut()
            .apply(f);
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

trait Apply {
    fn apply<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self);
}

impl Apply for EventStats {
    fn apply<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self),
    {
        f(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::MarketDataEvent;
    
    #[tokio::test]
    async fn test_publish_subscribe() {
        let bus = EventBus::new();
        
        // Subscribe before publishing
        let mut rx = bus.subscribe_market_data().await;
        
        // Publish event
        let event = Event::MarketData(MarketDataEvent {
            timestamp: 1234567890,
            symbol: "ES".to_string(),
            price: 6000.0,
            volume: 10.0,
            bid_price: 5999.5,
            bid_size: 5.0,
            ask_price: 6000.5,
            ask_size: 5.0,
        });
        
        bus.publish(event).await.unwrap();
        
        // Receive event
        let received = rx.recv().await.unwrap();
        assert_eq!(received.priority, 5);
    }
    
    #[tokio::test]
    async fn test_multiple_subscribers() {
        let bus = EventBus::new();
        
        let mut rx1 = bus.subscribe_market_data().await;
        let mut rx2 = bus.subscribe_market_data().await;
        
        let event = Event::MarketData(MarketDataEvent {
            timestamp: 1234567890,
            symbol: "ES".to_string(),
            price: 6000.0,
            volume: 10.0,
            bid_price: 5999.5,
            bid_size: 5.0,
            ask_price: 6000.5,
            ask_size: 5.0,
        });
        
        bus.publish(event).await.unwrap();
        
        // Both should receive
        assert!(rx1.recv().await.is_ok());
        assert!(rx2.recv().await.is_ok());
    }
}
