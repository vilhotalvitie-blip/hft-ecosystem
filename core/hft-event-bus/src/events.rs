//! Event type definitions for the HFT system

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Base event wrapper with metadata
#[derive(Debug, Clone)]
pub struct EventEnvelope {
    /// Unique event ID
    pub id: Uuid,
    
    /// Timestamp when event was created (nanoseconds)
    pub timestamp_ns: i64,
    
    /// Event priority (0 = highest)
    pub priority: u8,
    
    /// Event payload
    pub event: Box<dyn Event>,
}

impl EventEnvelope {
    pub fn new<T: Event + 'static>(event: T, priority: u8) -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        
        // Use monotonic counter instead of Uuid::new_v4() (avoids OS RNG syscall)
        let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
        // Construct a deterministic UUID from the counter (v4 format but no syscall)
        let id = Uuid::from_u128(seq as u128);
        
        Self {
            id,
            timestamp_ns: chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0),
            priority,
            event: Box::new(event),
        }
    }
}

// ============================================================================
// Market Data Events
// ============================================================================

/// Raw market data event (tick, quote, trade)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataEvent {
    pub timestamp: i64,
    pub symbol: String,
    pub price: f64,
    pub volume: f64,
    pub bid_price: f64,
    pub bid_size: f64,
    pub ask_price: f64,
    pub ask_size: f64,
}

/// Aggregated market data (candles, VWAP, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedDataEvent {
    pub timestamp: i64,
    pub symbol: String,
    pub timeframe: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub vwap: f64,
}

/// Order book snapshot or update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookEvent {
    pub timestamp: i64,
    pub symbol: String,
    pub bids: Vec<(f64, f64)>,  // (price, size)
    pub asks: Vec<(f64, f64)>,
}

/// Computed features for strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureEvent {
    pub timestamp: i64,
    pub symbol: String,
    pub features: std::collections::HashMap<String, f64>,
}

/// Quantum market analysis features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumFeatureEvent {
    pub timestamp: i64,
    pub symbol: String,
    
    // Quantum state metrics
    pub momentum: f64,
    pub variance: f64,
    pub skewness: f64,
    pub kurtosis: f64,
    
    // Regime detection
    pub regime: MarketRegime,
    pub regime_confidence: f64,
    
    // Liquidity analysis
    pub liquidity_walls: Vec<f64>,  // Price levels with walls
    pub wall_strengths: Vec<f64>,  // Curvature at each wall
    
    // Risk metrics
    pub momentum_uncertainty: f64,
    pub price_std_dev: f64,
    
    // Validation metrics (optional)
    pub validation_passed: Option<bool>,
    pub chi_squared_p_value: Option<f64>,
    pub kl_divergence: Option<f64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum MarketRegime {
    Consolidation,
    Trending,
    Volatile,
    RangeBound,
}

// ============================================================================
// Strategy & Signal Events
// ============================================================================

/// Trading signal from a strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalEvent {
    pub signal_id: Uuid,
    pub timestamp: i64,
    pub strategy_id: String,
    pub symbol: String,
    pub direction: SignalDirection,
    pub strength: f64,  // 0.0 to 1.0
    pub target_price: Option<f64>,
    pub stop_loss: Option<f64>,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SignalDirection {
    Long,
    Short,
    Neutral,
}

/// ML model prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionEvent {
    pub timestamp: i64,
    pub model_id: String,
    pub symbol: String,
    pub predicted_return: f64,
    pub confidence: f64,
}

/// Signal approved by risk management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovedSignalEvent {
    pub signal_id: Uuid,
    pub timestamp: i64,
    pub original_signal: SignalEvent,
    pub approved_size: f64,
}

/// Signal rejected by risk management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectedSignalEvent {
    pub signal_id: Uuid,
    pub timestamp: i64,
    pub original_signal: SignalEvent,
    pub rejection_reason: String,
}

// ============================================================================
// Execution Events
// ============================================================================

/// Order submitted to market
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderEvent {
    pub order_id: Uuid,
    pub signal_id: Option<Uuid>,
    pub timestamp: i64,
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub quantity: f64,
    pub price: Option<f64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum OrderType {
    Market,
    Limit,
    Stop,
    StopLimit,
}

/// Order filled (partial or complete)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FillEvent {
    pub fill_id: Uuid,
    pub order_id: Uuid,
    pub signal_id: Option<Uuid>,
    pub timestamp: i64,
    pub symbol: String,
    pub side: OrderSide,
    pub filled_quantity: f64,
    pub fill_price: f64,
    pub commission: f64,
    pub slippage_bps: f64,
}

/// Order status update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderUpdateEvent {
    pub order_id: Uuid,
    pub timestamp: i64,
    pub status: OrderStatus,
    pub filled_quantity: f64,
    pub remaining_quantity: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum OrderStatus {
    Pending,
    Submitted,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
}

// ============================================================================
// Metrics & Performance Events
// ============================================================================

/// Performance metrics update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsEvent {
    pub timestamp: i64,
    pub strategy_id: Option<String>,
    pub pnl: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub win_rate: f64,
    pub total_trades: u64,
}

/// System performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceEvent {
    pub timestamp: i64,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub event_latency_us: f64,
    pub events_per_second: f64,
}

// ============================================================================
// System Events
// ============================================================================

/// Configuration change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigEvent {
    pub timestamp: i64,
    pub config_key: String,
    pub old_value: Option<String>,
    pub new_value: String,
}

/// System health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthEvent {
    pub timestamp: i64,
    pub component: String,
    pub status: HealthStatus,
    pub message: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Error event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEvent {
    pub timestamp: i64,
    pub component: String,
    pub error_type: String,
    pub message: String,
    pub context: std::collections::HashMap<String, String>,
}

/// Research analysis event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchEvent {
    pub timestamp: i64,
    pub analysis_type: String,
    pub results: std::collections::HashMap<String, serde_json::Value>,
    pub confidence: f64,
}

// ============================================================================
// Trait Definitions for Event System
// ============================================================================

/// Base trait for all events in system
pub trait Event: Send + Sync + std::fmt::Debug {
    /// Get event type identifier
    fn event_type(&self) -> &'static str;
    
    /// Get event priority (0 = highest)
    fn priority(&self) -> u8 { 5 }
}

/// Trait for market-related events
pub trait MarketEvent: Event {
    /// Event timestamp
    fn timestamp(&self) -> i64;
    
    /// Associated symbol (if applicable)
    fn symbol(&self) -> Option<&str> { None }
    
    /// Event type classification
    fn event_type(&self) -> EventType {
        EventType::Custom
    }
}

/// Event type classification
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum EventType {
    Trade,
    Quote,
    OrderBook,
    Feature,
    Signal,
    Custom,
}

// ============================================================================
// Event Trait Implementations
// ============================================================================

impl Event for MarketDataEvent {
    fn event_type(&self) -> &'static str { "market_data" }
}

impl Event for AggregatedDataEvent {
    fn event_type(&self) -> &'static str { "aggregated_data" }
}

impl Event for FeatureEvent {
    fn event_type(&self) -> &'static str { "feature" }
}

impl Event for OrderBookEvent {
    fn event_type(&self) -> &'static str { "order_book" }
}

impl Event for QuantumFeatureEvent {
    fn event_type(&self) -> &'static str { "quantum" }
}

impl Event for SignalEvent {
    fn event_type(&self) -> &'static str { "signal" }
}

impl Event for OrderEvent {
    fn event_type(&self) -> &'static str { "order" }
}

impl Event for FillEvent {
    fn event_type(&self) -> &'static str { "fill" }
}

impl Event for OrderUpdateEvent {
    fn event_type(&self) -> &'static str { "order_update" }
}

impl Event for MetricsEvent {
    fn event_type(&self) -> &'static str { "metrics" }
}

impl Event for PerformanceEvent {
    fn event_type(&self) -> &'static str { "performance" }
}

impl Event for HealthEvent {
    fn event_type(&self) -> &'static str { "health" }
}

impl Event for ErrorEvent {
    fn event_type(&self) -> &'static str { "error" }
}
