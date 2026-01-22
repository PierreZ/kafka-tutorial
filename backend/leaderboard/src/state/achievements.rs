use serde::{Deserialize, Serialize};
use std::fmt;

/// Achievement types - step-based progress only
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AchievementType {
    // Step-based progress achievements
    Connected, // Step 1: Consumer group active
    FirstLoad, // Step 3: First message to actions topic
    Scaled,    // Step 4: 2+ consumers in group
    StatsDone, // Step 5: Message to team_stats topic
}

/// Emoji for incomplete step
pub const STEP_INCOMPLETE: &str = "⚪";

impl AchievementType {
    /// Get the emoji for this achievement
    pub fn emoji(&self) -> &'static str {
        match self {
            AchievementType::Connected => "🔌", // Plug = connection
            AchievementType::FirstLoad => "📤", // Outbox = producing
            AchievementType::Scaled => "⚖️",    // Balance = scaling
            AchievementType::StatsDone => "📊", // Chart = stats
        }
    }

    /// Get the name of this achievement
    pub fn name(&self) -> &'static str {
        match self {
            AchievementType::Connected => "Connected",
            AchievementType::FirstLoad => "First Load",
            AchievementType::Scaled => "Scaled",
            AchievementType::StatsDone => "Stats Published",
        }
    }

    /// Get a description explaining how to earn this achievement
    pub fn description(&self) -> &'static str {
        match self {
            AchievementType::Connected => "Start your consumer and join the consumer group",
            AchievementType::FirstLoad => "Successfully produce your first valid action message",
            AchievementType::Scaled => "Run 2+ consumers in your group (teaches parallelism)",
            AchievementType::StatsDone => "Produce a message to the team_stats topic",
        }
    }

    /// Check if this is a step achievement (all are now step achievements)
    pub fn is_step(&self) -> bool {
        true
    }

    /// Get all step achievements in display order
    pub fn all_steps() -> [AchievementType; 4] {
        [
            AchievementType::Connected,
            AchievementType::FirstLoad,
            AchievementType::Scaled,
            AchievementType::StatsDone,
        ]
    }
}

impl fmt::Display for AchievementType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.emoji(), self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_step() {
        assert!(AchievementType::Connected.is_step());
        assert!(AchievementType::FirstLoad.is_step());
        assert!(AchievementType::Scaled.is_step());
        assert!(AchievementType::StatsDone.is_step());
    }

    #[test]
    fn test_all_steps_count() {
        assert_eq!(AchievementType::all_steps().len(), 4);
    }
}
