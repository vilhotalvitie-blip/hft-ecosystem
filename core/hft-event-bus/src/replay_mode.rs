//! Event replay mode for backtesting and research
//!
//! Provides deterministic replay of historical events through the EventBus.
//! Strategies subscribe to the bus normally — they don't know if events are
//! live or replayed.
//!
//! ## Replay Speeds
//!
//! - `ReplaySpeed::Max` — as fast as possible (no sleeps)
//! - `ReplaySpeed::Realtime` — replay at original wall-clock speed
//! - `ReplaySpeed::Multiplier(n)` — n× real-time speed
//!
//! ## Example
//!
//! ```rust,ignore
//! use hft_event_bus::{EventBus, EventReplay, ReplaySpeed};
//!
//! let bus = EventBus::new();
//! let mut replay = EventReplay::new(bus.clone(), ReplaySpeed::Max);
//!
//! // Load events (pre-sorted by timestamp)
//! replay.load_events(events);
//!
//! // Run replay — publishes events through the bus
//! let stats = replay.run().await;
//! println!("Replayed {} events in {:?}", stats.events_replayed, stats.wall_time);
//! ```

use crate::events::{Event, EventEnvelope};
use crate::bus::EventBus;
use std::time::{Duration, Instant};
use tracing::{info, debug};

/// Replay speed control
#[derive(Debug, Clone)]
pub enum ReplaySpeed {
    /// As fast as possible (no sleeps)
    Max,
    /// Real-time replay (1:1 with original timestamps)
    Realtime,
    /// Multiplied speed (e.g., 10.0 = 10× real-time)
    Multiplier(f64),
}

impl ReplaySpeed {
    /// Parse from string: "max", "realtime", "10x", "2.5x"
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "max" => ReplaySpeed::Max,
            "realtime" | "1x" => ReplaySpeed::Realtime,
            other => {
                if let Some(num_str) = other.strip_suffix('x') {
                    if let Ok(mult) = num_str.parse::<f64>() {
                        return ReplaySpeed::Multiplier(mult);
                    }
                }
                ReplaySpeed::Max
            }
        }
    }
}

/// Virtual time tracker for replay
#[derive(Debug, Clone)]
pub struct VirtualClock {
    /// Current virtual time (nanoseconds)
    current_ns: i64,
    /// First event timestamp
    start_ns: i64,
    /// Last event timestamp
    end_ns: i64,
}

impl VirtualClock {
    pub fn new() -> Self {
        Self {
            current_ns: 0,
            start_ns: 0,
            end_ns: 0,
        }
    }

    /// Set time bounds from event stream
    pub fn set_bounds(&mut self, start_ns: i64, end_ns: i64) {
        self.start_ns = start_ns;
        self.end_ns = end_ns;
        self.current_ns = start_ns;
    }

    /// Advance to timestamp
    pub fn advance_to(&mut self, timestamp_ns: i64) {
        self.current_ns = timestamp_ns;
    }

    /// Current virtual time
    pub fn current(&self) -> i64 {
        self.current_ns
    }

    /// Progress as fraction [0.0, 1.0]
    pub fn progress(&self) -> f64 {
        if self.end_ns <= self.start_ns {
            return 1.0;
        }
        (self.current_ns - self.start_ns) as f64 / (self.end_ns - self.start_ns) as f64
    }
}

impl Default for VirtualClock {
    fn default() -> Self {
        Self::new()
    }
}

/// Replay statistics
#[derive(Debug, Clone)]
pub struct ReplayStats {
    /// Total events replayed
    pub events_replayed: usize,
    /// Wall-clock time for replay
    pub wall_time: Duration,
    /// Virtual time span (nanoseconds)
    pub virtual_time_span_ns: i64,
    /// Events per second (wall-clock)
    pub events_per_second: f64,
    /// Effective speed multiplier vs real-time
    pub effective_speed: f64,
}

/// Callback invoked after each event is published
pub type OnEventCallback = Box<dyn FnMut(usize, &EventEnvelope) + Send>;

/// Callback invoked periodically for progress reporting
pub type OnProgressCallback = Box<dyn FnMut(f64, usize) + Send>;

/// Event replayer — feeds historical events through the EventBus
pub struct EventReplay {
    bus: EventBus,
    speed: ReplaySpeed,
    events: Vec<EventEnvelope>,
    clock: VirtualClock,
    on_event: Option<OnEventCallback>,
    on_progress: Option<OnProgressCallback>,
    progress_interval: usize,
}

impl EventReplay {
    /// Create new replayer
    pub fn new(bus: EventBus, speed: ReplaySpeed) -> Self {
        Self {
            bus,
            speed,
            events: Vec::new(),
            clock: VirtualClock::new(),
            on_event: None,
            on_progress: None,
            progress_interval: 10_000,
        }
    }

    /// Load events for replay (must be sorted by timestamp_ns)
    pub fn load_events(&mut self, mut events: Vec<EventEnvelope>) {
        // Sort by timestamp to ensure chronological order
        events.sort_by_key(|e| e.timestamp_ns);

        if let (Some(first), Some(last)) = (events.first(), events.last()) {
            self.clock.set_bounds(first.timestamp_ns, last.timestamp_ns);
        }

        info!("Loaded {} events for replay", events.len());
        self.events = events;
    }

    /// Set callback invoked after each event
    pub fn on_event(&mut self, callback: OnEventCallback) {
        self.on_event = Some(callback);
    }

    /// Set progress callback (called every N events)
    pub fn on_progress(&mut self, interval: usize, callback: OnProgressCallback) {
        self.progress_interval = interval;
        self.on_progress = Some(callback);
    }

    /// Get virtual clock reference
    pub fn clock(&self) -> &VirtualClock {
        &self.clock
    }

    /// Get number of loaded events
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Run the replay — publishes all events through the bus
    pub async fn run(&mut self) -> ReplayStats {
        let total = self.events.len();
        if total == 0 {
            return ReplayStats {
                events_replayed: 0,
                wall_time: Duration::ZERO,
                virtual_time_span_ns: 0,
                events_per_second: 0.0,
                effective_speed: 0.0,
            };
        }

        let wall_start = Instant::now();
        let first_event_ns = self.events[0].timestamp_ns;
        let last_event_ns = self.events[total - 1].timestamp_ns;
        let virtual_span = last_event_ns - first_event_ns;

        info!(
            "Starting replay: {} events, virtual span {:.3}s, speed {:?}",
            total,
            virtual_span as f64 / 1e9,
            self.speed
        );

        // Take events out to avoid borrow issues
        let events = std::mem::take(&mut self.events);

        for (i, envelope) in events.iter().enumerate() {
            // Advance virtual clock
            self.clock.advance_to(envelope.timestamp_ns);

            // Speed control
            match &self.speed {
                ReplaySpeed::Max => { /* no delay */ }
                ReplaySpeed::Realtime | ReplaySpeed::Multiplier(_) => {
                    let multiplier = match &self.speed {
                        ReplaySpeed::Realtime => 1.0,
                        ReplaySpeed::Multiplier(m) => *m,
                        _ => unreachable!(),
                    };

                    if i > 0 {
                        let virtual_delta_ns = envelope.timestamp_ns - events[i - 1].timestamp_ns;
                        if virtual_delta_ns > 0 {
                            let wall_delay_ns = (virtual_delta_ns as f64 / multiplier) as u64;
                            if wall_delay_ns > 1_000_000 {
                                // Only sleep if > 1ms to avoid overhead
                                tokio::time::sleep(Duration::from_nanos(wall_delay_ns)).await;
                            }
                        }
                    }
                }
            }

            // Publish event through the bus
            if let Err(e) = self.bus.publish(&*envelope.event).await {
                debug!("Failed to publish event {}: {}", i, e);
            }

            // Per-event callback
            if let Some(ref mut cb) = self.on_event {
                cb(i, envelope);
            }

            // Progress callback
            if let Some(ref mut cb) = self.on_progress {
                if i > 0 && i % self.progress_interval == 0 {
                    let progress = self.clock.progress();
                    cb(progress, i);
                }
            }
        }

        let wall_time = wall_start.elapsed();
        let events_per_second = if wall_time.as_secs_f64() > 0.0 {
            total as f64 / wall_time.as_secs_f64()
        } else {
            0.0
        };
        let effective_speed = if wall_time.as_nanos() > 0 && virtual_span > 0 {
            virtual_span as f64 / wall_time.as_nanos() as f64
        } else {
            0.0
        };

        // Put events back
        self.events = events;

        let stats = ReplayStats {
            events_replayed: total,
            wall_time,
            virtual_time_span_ns: virtual_span,
            events_per_second,
            effective_speed,
        };

        info!(
            "Replay complete: {} events in {:.3}s ({:.0} events/sec, {:.1}× real-time)",
            stats.events_replayed,
            stats.wall_time.as_secs_f64(),
            stats.events_per_second,
            stats.effective_speed
        );

        stats
    }

    /// Run replay up to a specific virtual timestamp
    pub async fn run_until(&mut self, end_ns: i64) -> ReplayStats {
        // Filter events to only those before end_ns
        let original_len = self.events.len();
        let cutoff = self.events.partition_point(|e| e.timestamp_ns <= end_ns);

        // Temporarily truncate
        let remaining = self.events.split_off(cutoff);
        let stats = self.run().await;

        // Restore remaining events
        self.events.extend(remaining);

        stats
    }
}

/// Builder for constructing EventReplay with fluent API
pub struct EventReplayBuilder {
    bus: EventBus,
    speed: ReplaySpeed,
    events: Vec<EventEnvelope>,
    progress_interval: usize,
}

impl EventReplayBuilder {
    pub fn new(bus: EventBus) -> Self {
        Self {
            bus,
            speed: ReplaySpeed::Max,
            events: Vec::new(),
            progress_interval: 10_000,
        }
    }

    pub fn speed(mut self, speed: ReplaySpeed) -> Self {
        self.speed = speed;
        self
    }

    pub fn events(mut self, events: Vec<EventEnvelope>) -> Self {
        self.events = events;
        self
    }

    pub fn progress_interval(mut self, interval: usize) -> Self {
        self.progress_interval = interval;
        self
    }

    pub fn build(self) -> EventReplay {
        let mut replay = EventReplay::new(self.bus, self.speed);
        replay.progress_interval = self.progress_interval;
        if !self.events.is_empty() {
            replay.load_events(self.events);
        }
        replay
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::MarketDataEvent;

    fn make_envelope(ts_ns: i64, price: f64) -> EventEnvelope {
        let mut env = EventEnvelope::new(
            Event::MarketData(MarketDataEvent {
                timestamp: ts_ns,
                symbol: "ES".to_string(),
                price,
                volume: 1.0,
                bid_price: price - 0.125,
                bid_size: 10.0,
                ask_price: price + 0.125,
                ask_size: 10.0,
            }),
            5,
        );
        // Override envelope timestamp to match logical event time
        env.timestamp_ns = ts_ns;
        env
    }

    #[test]
    fn test_replay_speed_parse() {
        assert!(matches!(ReplaySpeed::from_str("max"), ReplaySpeed::Max));
        assert!(matches!(ReplaySpeed::from_str("realtime"), ReplaySpeed::Realtime));
        match ReplaySpeed::from_str("10x") {
            ReplaySpeed::Multiplier(m) => assert!((m - 10.0).abs() < 1e-10),
            _ => panic!("Expected Multiplier"),
        }
        match ReplaySpeed::from_str("2.5x") {
            ReplaySpeed::Multiplier(m) => assert!((m - 2.5).abs() < 1e-10),
            _ => panic!("Expected Multiplier"),
        }
    }

    #[test]
    fn test_virtual_clock() {
        let mut clock = VirtualClock::new();
        clock.set_bounds(1_000_000_000, 2_000_000_000);
        assert_eq!(clock.current(), 1_000_000_000);
        assert!((clock.progress() - 0.0).abs() < 1e-10);

        clock.advance_to(1_500_000_000);
        assert!((clock.progress() - 0.5).abs() < 1e-10);

        clock.advance_to(2_000_000_000);
        assert!((clock.progress() - 1.0).abs() < 1e-10);
    }

    #[tokio::test]
    async fn test_replay_max_speed() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe_market_data().await;

        let events: Vec<EventEnvelope> = (0..100)
            .map(|i| make_envelope(i * 1_000_000, 6000.0 + i as f64))
            .collect();

        let mut replay = EventReplay::new(bus, ReplaySpeed::Max);
        replay.load_events(events);

        assert_eq!(replay.event_count(), 100);

        let stats = replay.run().await;

        assert_eq!(stats.events_replayed, 100);
        assert!(stats.wall_time.as_millis() < 1000); // Should be very fast
        assert!(stats.events_per_second > 100.0);
    }

    #[tokio::test]
    async fn test_replay_builder() {
        let bus = EventBus::new();
        let _rx = bus.subscribe_market_data().await;

        let events: Vec<EventEnvelope> = (0..10)
            .map(|i| make_envelope(i * 1_000_000, 6000.0))
            .collect();

        let mut replay = EventReplayBuilder::new(bus)
            .speed(ReplaySpeed::Max)
            .events(events)
            .build();

        let stats = replay.run().await;
        assert_eq!(stats.events_replayed, 10);
    }

    #[tokio::test]
    async fn test_replay_empty() {
        let bus = EventBus::new();
        let mut replay = EventReplay::new(bus, ReplaySpeed::Max);
        let stats = replay.run().await;
        assert_eq!(stats.events_replayed, 0);
    }

    #[tokio::test]
    async fn test_replay_preserves_order() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe_market_data().await;

        // Events intentionally out of order — load_events should sort them
        let events = vec![
            make_envelope(3_000_000, 6003.0),
            make_envelope(1_000_000, 6001.0),
            make_envelope(2_000_000, 6002.0),
        ];

        let mut replay = EventReplay::new(bus, ReplaySpeed::Max);
        replay.load_events(events);

        let stats = replay.run().await;
        assert_eq!(stats.events_replayed, 3);

        // Verify chronological order
        let e1 = rx.recv().await.unwrap();
        let e2 = rx.recv().await.unwrap();
        let e3 = rx.recv().await.unwrap();

        match (&e1.event, &e2.event, &e3.event) {
            (Event::MarketData(m1), Event::MarketData(m2), Event::MarketData(m3)) => {
                assert!((m1.price - 6001.0).abs() < 1e-10);
                assert!((m2.price - 6002.0).abs() < 1e-10);
                assert!((m3.price - 6003.0).abs() < 1e-10);
            }
            _ => panic!("Expected MarketData events"),
        }
    }
}
