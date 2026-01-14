use super::achievements::AchievementType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::Instant;

/// Snapshot of team stats from the compacted topic
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TeamStatsSnapshot {
    pub processed: u64,
    pub flagged: u64,
}

/// State for a single team - simplified for step-based tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamState {
    pub team_name: String,
    pub achievements: HashSet<AchievementType>,
    pub action_count: u64, // Valid messages to actions topic
    pub stats_count: u64,  // Messages to team_stats topic
    pub error_counts: HashMap<AchievementType, u32>, // ParseError and MissingFields counts
    #[serde(default)]
    pub max_lag_seen: i64, // Peak lag ever observed (for LagBuster)
    #[serde(default)]
    pub current_lag: i64, // Current consumer lag
    #[serde(default)]
    pub messages_consumed: u64, // Total messages consumed from new_users topic (for accurate lag)
    #[serde(default)]
    pub clean_streak_count: u64, // Consecutive valid messages without errors (for CleanStreak)
    #[serde(default)]
    pub correct_key_count: u64, // Stats messages with correct key (for KeyMaster)
    #[serde(default)]
    pub achievement_timestamps: HashMap<AchievementType, DateTime<Utc>>, // When each achievement was unlocked
    #[serde(default)]
    pub session_start: Option<DateTime<Utc>>, // When this team's session started
    // Ephemeral fields for real-time metrics (not persisted)
    #[serde(skip)]
    pub last_action_time: Option<Instant>, // When last action was produced
    #[serde(skip)]
    pub last_stats_time: Option<Instant>, // When last stats message was produced
    #[serde(skip)]
    pub latest_stats: Option<TeamStatsSnapshot>, // Latest stats from compacted topic
    #[serde(skip)]
    pub recent_actions: VecDeque<Instant>, // Rolling window for production rate
}

impl TeamState {
    pub fn new(team_name: String) -> Self {
        Self {
            team_name,
            achievements: HashSet::new(),
            action_count: 0,
            stats_count: 0,
            error_counts: HashMap::new(),
            max_lag_seen: 0,
            current_lag: 0,
            messages_consumed: 0,
            clean_streak_count: 0,
            correct_key_count: 0,
            achievement_timestamps: HashMap::new(),
            session_start: None,
            last_action_time: None,
            last_stats_time: None,
            latest_stats: None,
            recent_actions: VecDeque::new(),
        }
    }

    /// Record a stats message, tracking correct key usage
    pub fn record_stats_message(&mut self, stats: TeamStatsSnapshot, key_correct: bool) {
        self.stats_count += 1;
        self.last_stats_time = Some(Instant::now());
        self.latest_stats = Some(stats);
        if key_correct {
            self.correct_key_count += 1;
        }
    }

    /// Unlock an achievement, returns true if newly unlocked
    /// Also records the timestamp when the achievement was first unlocked
    pub fn unlock_achievement(&mut self, achievement: AchievementType) -> bool {
        let is_new = self.achievements.insert(achievement);
        if is_new {
            self.achievement_timestamps.insert(achievement, Utc::now());
            // Set session start on first achievement if not already set
            if self.session_start.is_none() {
                self.session_start = Some(Utc::now());
            }
        }
        is_new
    }

    /// Check if team has a specific achievement
    pub fn has_achievement(&self, achievement: AchievementType) -> bool {
        self.achievements.contains(&achievement)
    }

    /// Record an error (increments count and marks as encountered)
    /// Also resets the clean streak counter
    pub fn record_error(&mut self, error: AchievementType) {
        if error.is_error() {
            let count = self.error_counts.entry(error).or_insert(0);
            *count = count.saturating_add(1);
            self.achievements.insert(error);
            self.clean_streak_count = 0; // Reset streak on any error
        }
    }

    /// Increment clean streak counter (called on valid message)
    pub fn increment_clean_streak(&mut self) {
        self.clean_streak_count = self.clean_streak_count.saturating_add(1);
    }

    /// Get error count for a specific error type
    pub fn get_error_count(&self, error: AchievementType) -> u32 {
        *self.error_counts.get(&error).unwrap_or(&0)
    }

    /// Count of step achievements earned (for sorting)
    pub fn step_count(&self) -> usize {
        self.achievements.iter().filter(|a| a.is_step()).count()
    }

    /// Update lag tracking and check for LagBuster achievement
    pub fn update_lag(&mut self, new_lag: i64) {
        self.current_lag = new_lag;
        if new_lag > self.max_lag_seen {
            self.max_lag_seen = new_lag;
        }
    }

    /// Check if team qualifies for LagBuster (had threshold+ lag, now caught up)
    /// Uses <= 5 threshold instead of == 0 to handle timing fluctuations
    pub fn qualifies_for_lag_buster(&self, threshold: i64) -> bool {
        self.max_lag_seen >= threshold && self.current_lag <= 5
    }

    /// Check if team has all achievements required for Champion
    pub fn has_all_champion_requirements(&self) -> bool {
        AchievementType::champion_requirements()
            .iter()
            .all(|a| self.achievements.contains(a))
    }

    /// Format session duration as human-readable string
    pub fn session_duration_display(&self) -> String {
        match self.session_start {
            Some(start) => {
                let duration = Utc::now().signed_duration_since(start);
                let mins = duration.num_minutes();
                if mins < 60 {
                    format!("{}m", mins)
                } else {
                    format!("{}h {}m", mins / 60, mins % 60)
                }
            }
            None => "-".to_string(),
        }
    }
}

/// Consumer group status for a team
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ConsumerGroupStatus {
    pub team_name: String,
    pub state: GroupState,
    pub members: u32,
    pub lag: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum GroupState {
    Active,
    Rebalancing,
    Empty,
    Unknown,
}

impl std::fmt::Display for GroupState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GroupState::Active => write!(f, "Active"),
            GroupState::Rebalancing => write!(f, "Rebalancing"),
            GroupState::Empty => write!(f, "Empty"),
            GroupState::Unknown => write!(f, "Unknown"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unlock_achievement() {
        let mut state = TeamState::new("team-1".to_string());

        // First unlock returns true
        assert!(state.unlock_achievement(AchievementType::Connected));
        // Second unlock returns false
        assert!(!state.unlock_achievement(AchievementType::Connected));
    }

    #[test]
    fn test_has_achievement() {
        let mut state = TeamState::new("team-1".to_string());
        assert!(!state.has_achievement(AchievementType::Connected));

        state.unlock_achievement(AchievementType::Connected);
        assert!(state.has_achievement(AchievementType::Connected));
    }

    #[test]
    fn test_record_error() {
        let mut state = TeamState::new("team-1".to_string());

        state.record_error(AchievementType::ParseError);
        state.record_error(AchievementType::ParseError);

        assert_eq!(state.get_error_count(AchievementType::ParseError), 2);
        assert!(state.has_achievement(AchievementType::ParseError));
    }

    #[test]
    fn test_step_count() {
        let mut state = TeamState::new("team-1".to_string());
        assert_eq!(state.step_count(), 0);

        state.unlock_achievement(AchievementType::Connected);
        state.unlock_achievement(AchievementType::FirstLoad);
        state.unlock_achievement(AchievementType::ParseError); // Error, not a step

        assert_eq!(state.step_count(), 2);
    }

    #[test]
    fn test_lag_buster_qualification() {
        let mut state = TeamState::new("team-1".to_string());
        let threshold = 100; // Default threshold
        assert!(!state.qualifies_for_lag_buster(threshold));

        // Build up lag
        state.update_lag(50);
        assert!(!state.qualifies_for_lag_buster(threshold)); // Not enough lag

        state.update_lag(150);
        assert!(!state.qualifies_for_lag_buster(threshold)); // Still has lag

        // Catch up (threshold is <= 5)
        state.update_lag(5);
        assert!(state.qualifies_for_lag_buster(threshold)); // Qualifies at threshold

        state.update_lag(0);
        assert!(state.qualifies_for_lag_buster(threshold)); // Also qualifies at 0

        state.update_lag(6);
        assert!(!state.qualifies_for_lag_buster(threshold)); // Above threshold, doesn't qualify
    }

    #[test]
    fn test_champion_requirements() {
        let mut state = TeamState::new("team-1".to_string());
        assert!(!state.has_all_champion_requirements());

        // Unlock all required achievements
        for achievement in AchievementType::champion_requirements() {
            state.unlock_achievement(achievement);
        }

        assert!(state.has_all_champion_requirements());
    }

    #[test]
    fn test_clean_streak_increment() {
        let mut state = TeamState::new("team-1".to_string());
        assert_eq!(state.clean_streak_count, 0);

        state.increment_clean_streak();
        assert_eq!(state.clean_streak_count, 1);

        state.increment_clean_streak();
        state.increment_clean_streak();
        assert_eq!(state.clean_streak_count, 3);
    }

    #[test]
    fn test_clean_streak_reset_on_error() {
        let mut state = TeamState::new("team-1".to_string());

        // Build up a streak
        for _ in 0..10 {
            state.increment_clean_streak();
        }
        assert_eq!(state.clean_streak_count, 10);

        // Error should reset the streak
        state.record_error(AchievementType::ParseError);
        assert_eq!(state.clean_streak_count, 0);
    }

    #[test]
    fn test_clean_streak_recovery_after_error() {
        let mut state = TeamState::new("team-1".to_string());

        // Build streak, error, rebuild
        for _ in 0..25 {
            state.increment_clean_streak();
        }
        state.record_error(AchievementType::MissingFields);
        assert_eq!(state.clean_streak_count, 0);

        // Can rebuild streak after error
        for _ in 0..50 {
            state.increment_clean_streak();
        }
        assert_eq!(state.clean_streak_count, 50);
    }

    #[test]
    fn test_clean_streak_threshold() {
        let mut state = TeamState::new("team-1".to_string());

        // Simulate 49 valid messages
        for _ in 0..49 {
            state.increment_clean_streak();
        }
        assert_eq!(state.clean_streak_count, 49);

        // 50th message should qualify for achievement threshold
        state.increment_clean_streak();
        assert_eq!(state.clean_streak_count, 50);
        assert!(state.clean_streak_count >= 50);
    }
}
