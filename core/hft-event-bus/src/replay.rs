//! Event recording and replay for debugging and backtesting

use crate::events::EventEnvelope;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Records events for replay
pub struct EventRecorder {
    /// Circular buffer of events
    events: Arc<RwLock<Vec<EventEnvelope>>>,
    
    /// Maximum capacity
    capacity: usize,
    
    /// Current write position
    position: Arc<RwLock<usize>>,
}

impl EventRecorder {
    /// Create new recorder with given capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::with_capacity(capacity))),
            capacity,
            position: Arc::new(RwLock::new(0)),
        }
    }
    
    /// Record an event
    pub async fn record(&self, event: EventEnvelope) {
        let mut events = self.events.write().await;
        let mut pos = self.position.write().await;
        
        if events.len() < self.capacity {
            events.push(event);
        } else {
            // Circular buffer - overwrite oldest
            events[*pos] = event;
        }
        
        *pos = (*pos + 1) % self.capacity;
    }
    
    /// Get all recorded events
    pub async fn get_events(&self) -> Vec<EventEnvelope> {
        self.events.read().await.clone()
    }
    
    /// Get events in time range
    pub async fn get_events_in_range(&self, start_ns: i64, end_ns: i64) -> Vec<EventEnvelope> {
        self.events.read().await
            .iter()
            .filter(|e| e.timestamp_ns >= start_ns && e.timestamp_ns <= end_ns)
            .cloned()
            .collect()
    }
    
    /// Clear all recorded events
    pub async fn clear(&self) {
        let mut events = self.events.write().await;
        let mut pos = self.position.write().await;
        events.clear();
        *pos = 0;
    }
    
    /// Get number of recorded events
    pub async fn len(&self) -> usize {
        self.events.read().await.len()
    }
    
    /// Check if recorder is empty
    pub async fn is_empty(&self) -> bool {
        self.events.read().await.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{Event, MarketDataEvent};
    
    #[tokio::test]
    async fn test_record_and_retrieve() {
        let recorder = EventRecorder::new(100);
        
        let event = EventEnvelope::new(
            Event::MarketData(MarketDataEvent {
                timestamp: 1234567890,
                symbol: "ES".to_string(),
                price: 6000.0,
                volume: 10.0,
                bid_price: 5999.5,
                bid_size: 5.0,
                ask_price: 6000.5,
                ask_size: 5.0,
            }),
            5,
        );
        
        recorder.record(event.clone()).await;
        
        let events = recorder.get_events().await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, event.id);
    }
    
    #[tokio::test]
    async fn test_circular_buffer() {
        let recorder = EventRecorder::new(2);
        
        for i in 0..5 {
            let event = EventEnvelope::new(
                Event::MarketData(MarketDataEvent {
                    timestamp: i,
                    symbol: "ES".to_string(),
                    price: 6000.0 + i as f64,
                    volume: 10.0,
                    bid_price: 5999.5,
                    bid_size: 5.0,
                    ask_price: 6000.5,
                    ask_size: 5.0,
                }),
                5,
            );
            recorder.record(event).await;
        }
        
        let events = recorder.get_events().await;
        assert_eq!(events.len(), 2); // Only keeps last 2
    }
}
