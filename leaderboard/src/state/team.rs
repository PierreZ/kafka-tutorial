use super::achievements::AchievementType;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// State for a single team - simplified for step-based tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamState {
    pub team_name: String,
    pub achievements: HashSet<AchievementType>,
    pub action_count: u64,    // Valid messages to actions topic
    pub watchlist_count: u64, // Messages to watchlist topic
    pub error_counts: HashMap<AchievementType, u32>, // ParseError and MissingFields counts
}

impl TeamState {
    pub fn new(team_name: String) -> Self {
        Self {
            team_name,
            achievements: HashSet::new(),
            action_count: 0,
            watchlist_count: 0,
            error_counts: HashMap::new(),
        }
    }

    /// Unlock an achievement, returns true if newly unlocked
    pub fn unlock_achievement(&mut self, achievement: AchievementType) -> bool {
        self.achievements.insert(achievement)
    }

    /// Check if team has a specific achievement
    pub fn has_achievement(&self, achievement: AchievementType) -> bool {
        self.achievements.contains(&achievement)
    }

    /// Record an error (increments count and marks as encountered)
    pub fn record_error(&mut self, error: AchievementType) {
        if error.is_error() {
            *self.error_counts.entry(error).or_insert(0) += 1;
            self.achievements.insert(error);
        }
    }

    /// Get error count for a specific error type
    pub fn get_error_count(&self, error: AchievementType) -> u32 {
        *self.error_counts.get(&error).unwrap_or(&0)
    }

    /// Count of step achievements earned (for sorting)
    pub fn step_count(&self) -> usize {
        self.achievements.iter().filter(|a| a.is_step()).count()
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
}
