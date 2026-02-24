//! Zero-allocation fast channels for MarketEvent types
//!
//! This module provides high-performance channels optimized for
//! zero-copy event transmission with <1μs latency.

use market_data_engine::types::{MarketEvent, TradeV2, QuoteV2};
use flume::{Sender, Receiver, bounded, unbounded};
use std::sync::Arc;

/// Fast channel for MarketEvent types
///
/// # Design
/// - Lock-free SPSC/MPSC channels
/// - Zero-copy transmission
/// - Bounded to prevent memory issues
///
/// # Example
/// ```
/// use hft_event_bus::fast_channel::FastChannel;
///
/// let channel = FastChannel::<TradeV2>::bounded(10_000);
/// channel.send(trade)?;
/// let trade = channel.recv()?;
/// ```
pub struct FastChannel<E: MarketEvent> {
    sender: Sender<E>,
    receiver: Receiver<E>,
}

impl<E: MarketEvent> FastChannel<E> {
    /// Create bounded channel (recommended for backpressure)
    pub fn bounded(capacity: usize) -> Self {
        let (sender, receiver) = bounded(capacity);
        Self { sender, receiver }
    }
    
    /// Create unbounded channel (use with caution)
    pub fn unbounded() -> Self {
        let (sender, receiver) = unbounded();
        Self { sender, receiver }
    }
    
    /// Send event (zero-copy)
    #[inline(always)]
    pub fn send(&self, event: E) -> Result<(), SendError<E>> {
        self.sender.send(event).map_err(|e| SendError(e.0))
    }
    
    /// Try to send without blocking
    #[inline(always)]
    pub fn try_send(&self, event: E) -> Result<(), TrySendError<E>> {
        self.sender.try_send(event).map_err(|e| match e {
            flume::TrySendError::Full(ev) => TrySendError::Full(ev),
            flume::TrySendError::Disconnected(ev) => TrySendError::Disconnected(ev),
        })
    }
    
    /// Receive event (blocking)
    #[inline(always)]
    pub fn recv(&self) -> Result<E, RecvError> {
        self.receiver.recv().map_err(|_| RecvError)
    }
    
    /// Try to receive without blocking
    #[inline(always)]
    pub fn try_recv(&self) -> Result<E, TryRecvError> {
        self.receiver.try_recv().map_err(|e| match e {
            flume::TryRecvError::Empty => TryRecvError::Empty,
            flume::TryRecvError::Disconnected => TryRecvError::Disconnected,
        })
    }
    
    /// Get sender clone
    pub fn sender(&self) -> Sender<E> {
        self.sender.clone()
    }
    
    /// Get receiver clone
    pub fn receiver(&self) -> Receiver<E> {
        self.receiver.clone()
    }
    
    /// Check if channel is empty
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.receiver.is_empty()
    }
    
    /// Get number of messages in channel
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.receiver.len()
    }
}

impl<E: MarketEvent> Clone for FastChannel<E> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            receiver: self.receiver.clone(),
        }
    }
}

/// Send error
#[derive(Debug)]
pub struct SendError<E>(pub E);

impl<E> std::fmt::Display for SendError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "channel disconnected")
    }
}

impl<E: std::fmt::Debug> std::error::Error for SendError<E> {}

/// Try send error
#[derive(Debug)]
pub enum TrySendError<E> {
    Full(E),
    Disconnected(E),
}

impl<E> std::fmt::Display for TrySendError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Full(_) => write!(f, "channel full"),
            Self::Disconnected(_) => write!(f, "channel disconnected"),
        }
    }
}

impl<E: std::fmt::Debug> std::error::Error for TrySendError<E> {}

/// Receive error
#[derive(Debug)]
pub struct RecvError;

impl std::fmt::Display for RecvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "channel disconnected")
    }
}

impl std::error::Error for RecvError {}

/// Try receive error
#[derive(Debug)]
pub enum TryRecvError {
    Empty,
    Disconnected,
}

impl std::fmt::Display for TryRecvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "channel empty"),
            Self::Disconnected => write!(f, "channel disconnected"),
        }
    }
}

impl std::error::Error for TryRecvError {}

/// Multi-producer, single-consumer channel for MarketEvent
pub struct MpscChannel<E: MarketEvent> {
    sender: Sender<E>,
    receiver: Receiver<E>,
}

impl<E: MarketEvent> MpscChannel<E> {
    /// Create new MPSC channel
    pub fn new(capacity: usize) -> Self {
        let (sender, receiver) = bounded(capacity);
        Self { sender, receiver }
    }
    
    /// Get sender clone (can be shared across threads)
    pub fn sender(&self) -> Sender<E> {
        self.sender.clone()
    }
    
    /// Get receiver (should only be used by one consumer)
    pub fn receiver(&self) -> &Receiver<E> {
        &self.receiver
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use market_data_engine::types::{Price, Quantity, Timestamp, InstrumentId, SideV2, TradeFlags};
    
    fn create_test_trade() -> TradeV2 {
        TradeV2 {
            timestamp: Timestamp::now(),
            instrument_id: InstrumentId::from_raw(1),
            price: Price::from_float(100.0),
            quantity: Quantity::new(10),
            side: SideV2::Buy,
            trade_id: 1,
            exchange: 1,
            flags: TradeFlags::new(0),
            _padding: [0; 12],
        }
    }
    
    #[test]
    fn test_fast_channel_send_recv() {
        let channel = FastChannel::<TradeV2>::bounded(100);
        let trade = create_test_trade();
        
        channel.send(trade).unwrap();
        let received = channel.recv().unwrap();
        
        assert_eq!(received.price, trade.price);
    }
    
    #[test]
    fn test_fast_channel_try_send() {
        let channel = FastChannel::<TradeV2>::bounded(1);
        let trade = create_test_trade();
        
        channel.try_send(trade).unwrap();
        
        // Channel full
        let result = channel.try_send(trade);
        assert!(matches!(result, Err(TrySendError::Full(_))));
    }
    
    #[test]
    fn test_fast_channel_empty() {
        let channel = FastChannel::<TradeV2>::bounded(100);
        assert!(channel.is_empty());
        
        channel.send(create_test_trade()).unwrap();
        assert!(!channel.is_empty());
        assert_eq!(channel.len(), 1);
    }
}
