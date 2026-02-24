//! # HFT Event Bus
//!
//! High-performance, type-safe event bus for HFT system communication.
//!
//! ## Features
//!
//! - **Type Safety**: Strongly typed events with compile-time guarantees
//! - **Performance**: Lock-free where possible, <10μs latency
//! - **Multi-threaded**: Safe concurrent pub/sub
//! - **Priority Queues**: Critical events processed first
//! - **Event Replay**: Debugging and backtesting support
//! - **Backpressure**: Bounded channels prevent memory issues
//!
//! ## Example
//!
//! ```rust
//! use hft_event_bus::{EventBus, Event, MarketDataEvent};
//!
//! #[tokio::main]
//! async fn main() {
//!     let bus = EventBus::new();
//!     
//!     // Subscribe to market data events
//!     let mut rx = bus.subscribe::<MarketDataEvent>().await;
//!     
//!     // Publish event
//!     bus.publish(Event::MarketData(MarketDataEvent {
//!         timestamp: 1234567890,
//!         symbol: "ES".to_string(),
//!         price: 6000.0,
//!         volume: 10.0,
//!     })).await;
//!     
//!     // Receive event
//!     if let Some(event) = rx.recv().await {
//!         println!("Received: {:?}", event);
//!     }
//! }
//! ```

// Legacy event system
pub mod events;
pub mod bus;
pub mod subscriber;
pub mod publisher;
pub mod replay;
pub mod replay_mode;

// New typed event system (zero-allocation)
pub mod fast_channel;
pub mod typed_bus;

// Research topic events
pub mod research_topic;

// Re-exports
pub use events::*;
pub use bus::EventBus;
pub use subscriber::Subscriber;
pub use publisher::Publisher;
pub use replay::EventRecorder;
pub use replay_mode::{EventReplay, EventReplayBuilder, ReplaySpeed, ReplayStats, VirtualClock};

// New typed exports
pub use fast_channel::FastChannel;
pub use typed_bus::TypedEventBus;

// Research topic exports
pub use research_topic::{ResearchEvent, SignalCreatedEvent, SignalUpdatedEvent, SignalDeletedEvent, AnalysisRequestedEvent, AnalysisStartedEvent, AnalysisProgressEvent, AnalysisCompletedEvent, AnalysisFailedEvent, FeatureExtractedEvent, FeaturePipelineUpdatedEvent, ModelTrainingStartedEvent, ModelTrainingProgressEvent, ModelTrainingCompletedEvent, ModelDeploymentRequestedEvent, ModelDeploymentCompletedEvent, RealTimeDataUpdateEvent, VisualizationUpdateEvent, StatisticalTestCompletedEvent, CorrelationMatrixUpdatedEvent, ResearchConfigUpdatedEvent, ResearchStateChangedEvent};
