//! Typed event bus for MarketEvent types
//!
//! This module provides a type-safe event bus that works with
//! the MarketEvent trait for zero-allocation event processing.

use market_data_engine::types::{MarketEvent, EventType, TradeV2, QuoteV2};
use crate::fast_channel::{FastChannel, SendError};
use dashmap::DashMap;
use std::sync::Arc;
use std::any::TypeId;

/// Typed event bus for MarketEvent types
///
/// # Design
/// - Type-safe channels per event type
/// - Zero-allocation transmission
/// - Lock-free concurrent access
///
/// # Example
/// ```
/// use hft_event_bus::typed_bus::TypedEventBus;
///
/// let bus = TypedEventBus::new();
///
/// // Subscribe to trades
/// let trade_rx = bus.subscribe::<TradeV2>();
///
/// // Publish trade
/// bus.publish(trade)?;
///
/// // Receive trade
/// let trade = trade_rx.recv()?;
/// ```
pub struct TypedEventBus {
    /// Channels indexed by TypeId
    channels: Arc<DashMap<TypeId, Arc<dyn std::any::Any + Send + Sync>>>,
    
    /// Statistics
    stats: Arc<DashMap<TypeId, TypedEventStats>>,
}

#[derive(Debug, Clone, Default)]
pub struct TypedEventStats {
    pub published: u64,
    pub received: u64,
    pub subscribers: usize,
}

impl TypedEventBus {
    /// Create new typed event bus
    pub fn new() -> Self {
        Self {
            channels: Arc::new(DashMap::new()),
            stats: Arc::new(DashMap::new()),
        }
    }
    
    /// Publish event (zero-copy)
    #[inline]
    pub fn publish<E: MarketEvent>(&self, event: E) -> Result<(), SendError<E>> {
        let type_id = TypeId::of::<E>();
        
        // Get or create channel
        let channel = self.get_or_create_channel::<E>();
        
        // Send event
        let result = channel.send(event);
        
        // Update stats
        if result.is_ok() {
            self.stats.entry(type_id)
                .or_insert_with(TypedEventStats::default)
                .published += 1;
        }
        
        result
    }
    
    /// Subscribe to event type
    pub fn subscribe<E: MarketEvent>(&self) -> flume::Receiver<E> {
        let type_id = TypeId::of::<E>();
        let channel = self.get_or_create_channel::<E>();
        
        // Update subscriber count
        self.stats.entry(type_id)
            .or_insert_with(TypedEventStats::default)
            .subscribers += 1;
        
        channel.receiver()
    }
    
    /// Get or create channel for event type
    fn get_or_create_channel<E: MarketEvent>(&self) -> Arc<FastChannel<E>> {
        let type_id = TypeId::of::<E>();
        
        let arc_any = self.channels.entry(type_id)
            .or_insert_with(|| {
                let channel = FastChannel::<E>::bounded(100_000);
                Arc::new(channel) as Arc<dyn std::any::Any + Send + Sync>
            })
            .clone();
        
        arc_any.downcast::<FastChannel<E>>()
            .expect("Type mismatch in channel registry")
    }
    
    /// Get statistics for event type
    pub fn stats<E: MarketEvent>(&self) -> Option<TypedEventStats> {
        let type_id = TypeId::of::<E>();
        self.stats.get(&type_id).map(|s| s.clone())
    }
    
    /// Get total events published across all types
    pub fn total_published(&self) -> u64 {
        self.stats.iter().map(|s| s.published).sum()
    }
}

impl Default for TypedEventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for TypedEventBus {
    fn clone(&self) -> Self {
        Self {
            channels: self.channels.clone(),
            stats: self.stats.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use market_data_engine::types::{Price, Quantity, Timestamp, InstrumentId, SideV2, TradeFlags};
    
    fn create_test_trade(id: u64) -> TradeV2 {
        TradeV2 {
            timestamp: Timestamp::from_nanos(id as i64),
            instrument_id: InstrumentId::from_raw(1),
            price: Price::from_float(100.0),
            quantity: Quantity::new(10),
            side: SideV2::Buy,
            trade_id: id,
            exchange: 1,
            flags: TradeFlags::new(0),
            _padding: [0; 12],
        }
    }
    
    fn create_test_quote(id: u64) -> QuoteV2 {
        QuoteV2 {
            timestamp: Timestamp::from_nanos(id as i64),
            instrument_id: InstrumentId::from_raw(1),
            bid_price: Price::from_float(100.0),
            ask_price: Price::from_float(100.25),
            bid_size: Quantity::new(10),
            ask_size: Quantity::new(10),
            exchange: 1,
            _padding: [0; 14],
        }
    }
    
    #[test]
    fn test_typed_bus_publish_subscribe() {
        let bus = TypedEventBus::new();
        
        // Subscribe
        let rx = bus.subscribe::<TradeV2>();
        
        // Publish
        let trade = create_test_trade(1);
        bus.publish(trade).unwrap();
        
        // Receive
        let received = rx.recv().unwrap();
        assert_eq!(received.trade_id, 1);
    }
    
    #[test]
    fn test_typed_bus_multiple_types() {
        let bus = TypedEventBus::new();
        
        // Subscribe to both types
        let trade_rx = bus.subscribe::<TradeV2>();
        let quote_rx = bus.subscribe::<QuoteV2>();
        
        // Publish both
        bus.publish(create_test_trade(1)).unwrap();
        bus.publish(create_test_quote(2)).unwrap();
        
        // Receive both
        let trade = trade_rx.recv().unwrap();
        let quote = quote_rx.recv().unwrap();
        
        assert_eq!(trade.trade_id, 1);
        assert_eq!(quote.timestamp.nanos(), 2);
    }
    
    #[test]
    fn test_typed_bus_stats() {
        let bus = TypedEventBus::new();
        let _rx = bus.subscribe::<TradeV2>();
        
        bus.publish(create_test_trade(1)).unwrap();
        bus.publish(create_test_trade(2)).unwrap();
        
        let stats = bus.stats::<TradeV2>().unwrap();
        assert_eq!(stats.published, 2);
        assert_eq!(stats.subscribers, 1);
    }
    
    #[test]
    fn test_typed_bus_clone() {
        let bus = TypedEventBus::new();
        let bus2 = bus.clone();
        
        let rx = bus.subscribe::<TradeV2>();
        
        // Publish from cloned bus
        bus2.publish(create_test_trade(1)).unwrap();
        
        // Receive from original
        let trade = rx.recv().unwrap();
        assert_eq!(trade.trade_id, 1);
    }
}
