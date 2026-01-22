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
    #[serde(default)]
    pub current_lag: i64, // Current consumer lag
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
            current_lag: 0,
            achievement_timestamps: HashMap::new(),
            session_start: None,
            last_action_time: None,
            last_stats_time: None,
            latest_stats: None,
            recent_actions: VecDeque::new(),
        }
    }

    /// Record a stats message
    pub fn record_stats_message(&mut self, stats: TeamStatsSnapshot) {
        self.stats_count += 1;
        self.last_stats_time = Some(Instant::now());
        self.latest_stats = Some(stats);
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

    /// Count of step achievements earned (for sorting)
    pub fn step_count(&self) -> usize {
        self.achievements.iter().filter(|a| a.is_step()).count()
    }

    /// Update lag tracking
    pub fn update_lag(&mut self, new_lag: i64) {
        self.current_lag = new_lag;
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
    fn test_step_count() {
        let mut state = TeamState::new("team-1".to_string());
        assert_eq!(state.step_count(), 0);

        state.unlock_achievement(AchievementType::Connected);
        state.unlock_achievement(AchievementType::FirstLoad);

        assert_eq!(state.step_count(), 2);
    }

    #[test]
    fn test_update_lag() {
        let mut state = TeamState::new("team-1".to_string());
        assert_eq!(state.current_lag, 0);

        state.update_lag(50);
        assert_eq!(state.current_lag, 50);

        state.update_lag(150);
        assert_eq!(state.current_lag, 150);
    }
}
