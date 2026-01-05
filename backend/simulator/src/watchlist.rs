use dashmap::DashMap;
use std::sync::atomic::{AtomicU32, Ordering};

/// Tracks flag counts per company for watchlist functionality.
/// Thread-safe for concurrent access from multiple consumer instances.
pub struct WatchlistState {
    /// Map from company_name to flag_count
    company_flags: DashMap<String, AtomicU32>,
    /// Threshold for producing to watchlist (default: 3)
    threshold: u32,
}

impl WatchlistState {
    pub fn new(threshold: u32) -> Self {
        Self {
            company_flags: DashMap::new(),
            threshold,
        }
    }

    /// Increment flag count for a company.
    /// Returns Some(count) if threshold is reached or exceeded, None otherwise.
    pub fn increment(&self, company_name: &str) -> Option<u32> {
        let entry = self
            .company_flags
            .entry(company_name.to_string())
            .or_insert_with(|| AtomicU32::new(0));

        let new_count = entry.fetch_add(1, Ordering::SeqCst) + 1;

        if new_count >= self.threshold {
            Some(new_count)
        } else {
            None
        }
    }
}
