use std::sync::atomic::{AtomicU64, Ordering};

/// Tracks processing statistics for a team.
/// Thread-safe for concurrent access from multiple consumer instances.
pub struct StatsState {
    processed: AtomicU64,
    flagged: AtomicU64,
}

impl StatsState {
    pub fn new() -> Self {
        Self {
            processed: AtomicU64::new(0),
            flagged: AtomicU64::new(0),
        }
    }

    /// Increment processed count and return new value.
    pub fn increment_processed(&self) -> u64 {
        self.processed.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// Increment flagged count and return (processed, flagged) tuple.
    pub fn increment_flagged(&self) -> (u64, u64) {
        let flagged = self.flagged.fetch_add(1, Ordering::SeqCst) + 1;
        let processed = self.processed.load(Ordering::SeqCst);
        (processed, flagged)
    }
}

impl Default for StatsState {
    fn default() -> Self {
        Self::new()
    }
}
