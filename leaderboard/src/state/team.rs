use super::achievements::{check_progress_achievements, Achievement, AchievementType};
use crate::validation::{ErrorType, ValidationResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// State for a single team
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamState {
    pub team_name: String,
    pub score: u64,
    pub correct_count: u64,
    pub incorrect_count: u64,
    pub current_streak: u32,
    pub best_streak: u32,
    pub achievements: Vec<Achievement>,
    pub error_counts: HashMap<ErrorType, u32>,
}

impl TeamState {
    pub fn new(team_name: String) -> Self {
        Self {
            team_name,
            score: 0,
            correct_count: 0,
            incorrect_count: 0,
            current_streak: 0,
            best_streak: 0,
            achievements: Vec::new(),
            error_counts: HashMap::new(),
        }
    }

    /// Process a validation result and return any new achievements unlocked
    pub fn process_result(&mut self, result: &ValidationResult) -> Vec<AchievementType> {
        match result {
            ValidationResult::Correct { points } => {
                self.correct_count += 1;
                self.score += *points as u64;
                self.current_streak += 1;
                if self.current_streak > self.best_streak {
                    self.best_streak = self.current_streak;
                }

                // Check for new progress achievements
                let new_achievements =
                    check_progress_achievements(self.correct_count, self.current_streak, &self.achievements);

                for achievement_type in &new_achievements {
                    self.score += achievement_type.points() as u64;
                    self.achievements.push(Achievement::new(*achievement_type));
                }

                new_achievements
            }
            ValidationResult::Incorrect { error } => {
                self.incorrect_count += 1;
                self.current_streak = 0; // Reset streak on error

                // Track error counts
                *self.error_counts.entry(error.error_type).or_insert(0) += 1;

                // Check for mistake achievements (first occurrence)
                let mut new_achievements = Vec::new();
                let mistake_achievement = match error.error_type {
                    ErrorType::InvalidJson => Some(AchievementType::ParseError),
                    ErrorType::NonExistentUser => Some(AchievementType::GhostUser),
                    ErrorType::DuplicateAction => Some(AchievementType::Duplicate),
                    ErrorType::MissingFields => Some(AchievementType::MissingFields),
                    ErrorType::FalsePositive => Some(AchievementType::FalsePositive),
                    _ => None,
                };

                if let Some(achievement_type) = mistake_achievement {
                    let already_has = self
                        .achievements
                        .iter()
                        .any(|a| a.achievement_type == achievement_type);
                    if !already_has {
                        self.achievements.push(Achievement::new(achievement_type));
                        new_achievements.push(achievement_type);
                    }
                }

                new_achievements
            }
        }
    }

    /// Add an infrastructure achievement if not already unlocked
    pub fn add_infrastructure_achievement(&mut self, achievement_type: AchievementType) -> bool {
        let already_has = self
            .achievements
            .iter()
            .any(|a| a.achievement_type == achievement_type);
        if !already_has {
            self.score += achievement_type.points() as u64;
            self.achievements.push(Achievement::new(achievement_type));
            true
        } else {
            false
        }
    }

    /// Get total number of actions (correct + incorrect)
    pub fn total_actions(&self) -> u64 {
        self.correct_count + self.incorrect_count
    }

    /// Get accuracy percentage
    pub fn accuracy(&self) -> f64 {
        if self.total_actions() == 0 {
            0.0
        } else {
            (self.correct_count as f64 / self.total_actions() as f64) * 100.0
        }
    }
}

/// Consumer group status for a team
#[derive(Debug, Clone)]
pub struct ConsumerGroupStatus {
    pub team_name: String,
    pub state: GroupState,
    pub members: u32,
    pub lag: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    use crate::validation::ValidationError;

    #[test]
    fn test_team_state_correct_action() {
        let mut state = TeamState::new("team-1".to_string());

        let result = ValidationResult::Correct { points: 10 };
        let achievements = state.process_result(&result);

        assert_eq!(state.correct_count, 1);
        assert_eq!(state.current_streak, 1);
        assert!(achievements.contains(&AchievementType::FirstSteps));
    }

    #[test]
    fn test_team_state_incorrect_action() {
        let mut state = TeamState::new("team-1".to_string());
        state.current_streak = 5;

        let result = ValidationResult::Incorrect {
            error: ValidationError::new(ErrorType::InvalidJson, "Bad JSON"),
        };
        let achievements = state.process_result(&result);

        assert_eq!(state.incorrect_count, 1);
        assert_eq!(state.current_streak, 0); // Streak reset
        assert!(achievements.contains(&AchievementType::ParseError));
    }

    #[test]
    fn test_accuracy() {
        let mut state = TeamState::new("team-1".to_string());
        state.correct_count = 9;
        state.incorrect_count = 1;

        assert!((state.accuracy() - 90.0).abs() < 0.001);
    }
}
