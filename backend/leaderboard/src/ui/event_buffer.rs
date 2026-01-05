use kafka_common::{Action, WatchlistEntry};
use std::collections::VecDeque;
use std::time::Instant;

/// Parsed event data for efficient rendering without re-parsing
#[derive(Debug, Clone)]
pub enum ParsedEventData {
    /// new_users: Store pretty-printed JSON for display
    NewUser { pretty_json: String },
    /// actions: Parsed Action with validation status
    Action { action: Action, is_valid: bool },
    /// watchlist: Parsed WatchlistEntry
    Watchlist(WatchlistEntry),
    /// Failed to parse
    Invalid,
}

/// Single buffered event with metadata
#[derive(Debug, Clone)]
pub struct BufferedEvent {
    /// Raw JSON string (compact, single line)
    pub json: String,
    /// Team identifier if extractable (None for new_users)
    pub team: Option<String>,
    /// When the event was received
    pub received_at: Instant,
    /// Parsed data for efficient rendering
    pub parsed: Option<ParsedEventData>,
}

impl BufferedEvent {
    /// Constructor for new_users events
    pub fn new_user(compact_json: String, pretty_json: String) -> Self {
        Self {
            json: compact_json,
            team: None,
            received_at: Instant::now(),
            parsed: Some(ParsedEventData::NewUser { pretty_json }),
        }
    }

    /// Constructor for valid/invalid action events
    pub fn action(compact_json: String, action: Action, is_valid: bool) -> Self {
        let team = Some(action.team.clone());
        Self {
            json: compact_json,
            team,
            received_at: Instant::now(),
            parsed: Some(ParsedEventData::Action { action, is_valid }),
        }
    }

    /// Constructor for unparseable action events
    pub fn invalid_action(compact_json: String, team: Option<String>) -> Self {
        Self {
            json: compact_json,
            team,
            received_at: Instant::now(),
            parsed: Some(ParsedEventData::Invalid),
        }
    }

    /// Constructor for watchlist events
    pub fn watchlist(compact_json: String, entry: WatchlistEntry) -> Self {
        let team = Some(entry.team.clone());
        Self {
            json: compact_json,
            team,
            received_at: Instant::now(),
            parsed: Some(ParsedEventData::Watchlist(entry)),
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

    /// Get event by reverse index (0 = newest, len-1 = oldest)
    pub fn get_rev(&self, index: usize) -> Option<&BufferedEvent> {
        let len = self.events.len();
        if index < len {
            self.events.get(len - 1 - index)
        } else {
            None
        }
    }
}
