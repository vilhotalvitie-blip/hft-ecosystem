# HFT Event Bus

High-performance, type-safe event bus for HFT system inter-component communication.

## Features

- ✅ **Type Safety**: Strongly typed events with compile-time guarantees
- ✅ **Performance**: Lock-free where possible, <10μs latency target
- ✅ **Multi-threaded**: Safe concurrent pub/sub using Tokio broadcast channels
- ✅ **Priority Queues**: Critical events can be prioritized
- ✅ **Event Replay**: Record and replay events for debugging/backtesting
- ✅ **Backpressure**: Bounded channels (10k capacity) prevent memory issues
- ✅ **Statistics**: Track published/received/dropped events per type

## Quick Start

```rust
use hft_event_bus::{EventBus, Event, MarketDataEvent};

#[tokio::main]
async fn main() {
    // Create event bus
    let bus = EventBus::new();
    
    // Subscribe to market data
    let mut rx = bus.subscribe_market_data().await;
    
    // Publish event
    bus.publish(Event::MarketData(MarketDataEvent {
        timestamp: 1234567890,
        symbol: "ES".to_string(),
        price: 6000.0,
        volume: 10.0,
        bid_price: 5999.5,
        bid_size: 5.0,
        ask_price: 6000.5,
        ask_size: 5.0,
    })).await.unwrap();
    
    // Receive event
    if let Ok(envelope) = rx.recv().await {
        println!("Received: {:?}", envelope.event);
    }
}
```

## Event Types

### Market Data
- `MarketDataEvent` - Raw tick/quote data
- `AggregatedDataEvent` - Candles, VWAP, etc.
- `OrderBookEvent` - Order book snapshots
- `FeatureEvent` - Computed features

### Strategy & Signals
- `SignalEvent` - Trading signals
- `PredictionEvent` - ML predictions
- `ApprovedSignalEvent` - Risk-approved signals
- `RejectedSignalEvent` - Risk-rejected signals

### Execution
- `OrderEvent` - Order submissions
- `FillEvent` - Order fills
- `OrderUpdateEvent` - Order status updates

### Metrics
- `MetricsEvent` - Trading metrics
- `PerformanceEvent` - System performance

### System
- `ConfigEvent` - Configuration changes
- `HealthEvent` - Health checks
- `ErrorEvent` - Error notifications

## Architecture

```
Publisher → EventBus → Subscriber(s)
              ↓
         EventRecorder (optional)
```

## Performance

- **Latency**: <10μs p99 (target)
- **Throughput**: 1M+ events/sec
- **Memory**: Bounded channels prevent unbounded growth
- **Concurrency**: Lock-free DashMap for channel registry

## Usage in HFT System

```rust
// In market-data-engine
let bus = Arc::new(EventBus::new());
bus.publish(Event::MarketData(tick_data)).await?;

// In feature-pipeline
let mut rx = bus.subscribe_market_data().await;
while let Ok(envelope) = rx.recv().await {
    // Process market data, compute features
    bus.publish(Event::Feature(features)).await?;
}

// In strategy-engine
let mut rx = bus.subscribe_features().await;
while let Ok(envelope) = rx.recv().await {
    // Generate signals
    bus.publish(Event::Signal(signal)).await?;
}

// In hft-dashboard
let mut rx = bus.subscribe_market_data().await;
while let Ok(envelope) = rx.recv().await {
    // Update UI
}
```

## Event Recording

```rust
// Enable recording
let bus = EventBus::with_recording(100_000);

// Publish events...

// Replay for debugging
if let Some(recorder) = bus.recorder() {
    let events = recorder.get_events().await;
    for event in events {
        println!("{:?}", event);
    }
}
```

## Statistics

```rust
let stats = bus.get_stats();
for (event_type, stats) in stats {
    println!("{}: published={}, received={}, dropped={}",
        event_type, stats.published, stats.received, stats.dropped);
}
```

## Thread Safety

All operations are thread-safe:
- `EventBus` can be shared via `Arc`
- Publishers and subscribers can be on different threads
- No locks in hot path (uses broadcast channels)

## Integration

Add to `Cargo.toml`:

```toml
[dependencies]
hft-event-bus = { path = "../hft-event-bus" }
```
