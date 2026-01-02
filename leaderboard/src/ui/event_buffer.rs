use std::collections::VecDeque;
use std::time::Instant;

/// Single buffered event with metadata
#[derive(Debug, Clone)]
pub struct BufferedEvent {
    /// Raw JSON string (compact, single line)
    pub json: String,
    /// Team identifier if extractable (None for new_users)
    pub team: Option<String>,
    /// When the event was received
    pub received_at: Instant,
}

impl BufferedEvent {
    pub fn new(json: String, team: Option<String>) -> Self {
        Self {
            json,
            team,
            received_at: Instant::now(),
        }
    }

    /// Format relative time ("2s", "1m", "5m+")
    pub fn relative_time(&self) -> String {
        let elapsed = self.received_at.elapsed().as_secs();
        if elapsed < 60 {
            format!("{}s", elapsed)
        } else if elapsed < 300 {
            format!("{}m", elapsed / 60)
        } else {
            "5m+".to_string()
        }
    }
}

/// Ring buffer for events with configurable capacity
#[derive(Debug)]
pub struct EventBuffer {
    events: VecDeque<BufferedEvent>,
    capacity: usize,
}

impl EventBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            events: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Add event, evicting oldest if at capacity
    pub fn push(&mut self, event: BufferedEvent) {
        if self.events.len() >= self.capacity {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    /// Get events in reverse order (newest first for display)
    pub fn iter_rev(&self) -> impl Iterator<Item = &BufferedEvent> {
        self.events.iter().rev()
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}
