use serde::{Deserialize, Serialize};
use std::fmt;

/// Achievement types - minimal set for tracking tutorial progress
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AchievementType {
    // Step-based progress achievements
    Connected, // Step 1: Consumer group active
    FirstLoad, // Step 3: First message to actions topic
    Scaled,    // Step 4: 2+ consumers in group
    StatsDone, // Step 5: Message to team_stats topic

    // Error achievements (educational feedback)
    ParseError,    // Invalid JSON
    MissingFields, // Missing required fields

    // Bonus achievements
    PartitionExplorer, // 3+ consumers (teaches partition limits parallelism)
    HighThroughput,    // 100+ valid actions
    CleanStreak,       // 50+ messages with 0 errors
    FirstBlood,        // First team to get Connected
    LagBuster,         // Build up 100+ lag, then consume it all
    KeyMaster,         // 25+ stats messages with correct key
    StatsFirstBlood,   // First team to publish stats
    Champion,          // First team to complete ALL achievements
}

/// Emoji for incomplete step
pub const STEP_INCOMPLETE: &str = "⚪";

impl AchievementType {
    /// Get the emoji for this achievement
    pub fn emoji(&self) -> &'static str {
        match self {
            // Step emojis - distinct visual for each tutorial step
            AchievementType::Connected => "🔌", // Plug = connection
            AchievementType::FirstLoad => "📤", // Outbox = producing
            AchievementType::Scaled => "⚖️",    // Balance = scaling
            AchievementType::StatsDone => "📊", // Chart = stats
            // Error indicators
            AchievementType::ParseError => "🔴",
            AchievementType::MissingFields => "🟠",
            // Bonus achievements
            AchievementType::PartitionExplorer => "🔬",
            AchievementType::HighThroughput => "📈",
            AchievementType::CleanStreak => "✨",
            AchievementType::FirstBlood => "⚔️",
            AchievementType::LagBuster => "🚀",
            AchievementType::KeyMaster => "🔑",
            AchievementType::StatsFirstBlood => "🎯",
            AchievementType::Champion => "🏆",
        }
    }

    /// Get the name of this achievement
    pub fn name(&self) -> &'static str {
        match self {
            AchievementType::Connected => "Connected",
            AchievementType::FirstLoad => "First Load",
            AchievementType::Scaled => "Scaled",
            AchievementType::StatsDone => "Stats Published",
            AchievementType::ParseError => "Parse Error",
            AchievementType::MissingFields => "Missing Fields",
            AchievementType::PartitionExplorer => "Partition Explorer",
            AchievementType::HighThroughput => "High Throughput",
            AchievementType::CleanStreak => "Clean Streak",
            AchievementType::FirstBlood => "First Blood",
            AchievementType::LagBuster => "Lag Buster",
            AchievementType::KeyMaster => "Key Master",
            AchievementType::StatsFirstBlood => "Stats First",
            AchievementType::Champion => "Champion",
        }
    }

    /// Get a description explaining how to earn this achievement
    pub fn description(&self) -> &'static str {
        match self {
            AchievementType::Connected => "Start your consumer and join the consumer group",
            AchievementType::FirstLoad => "Successfully produce your first valid action message",
            AchievementType::Scaled => "Run 2+ consumers in your group (teaches parallelism)",
            AchievementType::StatsDone => "Produce a message to the team_stats topic",
            AchievementType::ParseError => "Produced invalid JSON (fix your serialization!)",
            AchievementType::MissingFields => "Produced JSON with missing required fields",
            AchievementType::PartitionExplorer => "Run 3+ consumers (teaches partition limits)",
            AchievementType::HighThroughput => "Produce 100+ valid action messages",
            AchievementType::CleanStreak => "Produce 50+ consecutive messages with zero errors",
            AchievementType::FirstBlood => "First team to get the Connected achievement",
            AchievementType::LagBuster => "Build up 100+ lag, then consume it all",
            AchievementType::KeyMaster => "Produce 25+ stats messages with correct key",
            AchievementType::StatsFirstBlood => "First team to publish stats",
            AchievementType::Champion => "First team to earn all other achievements",
        }
    }

    /// Check if this is a step achievement (progress)
    pub fn is_step(&self) -> bool {
        matches!(
            self,
            AchievementType::Connected
                | AchievementType::FirstLoad
                | AchievementType::Scaled
                | AchievementType::StatsDone
        )
    }

    /// Check if this is an error achievement
    pub fn is_error(&self) -> bool {
        matches!(
            self,
            AchievementType::ParseError | AchievementType::MissingFields
        )
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

    /// Get all bonus achievements in display order
    pub fn all_bonus() -> [AchievementType; 8] {
        [
            AchievementType::PartitionExplorer,
            AchievementType::HighThroughput,
            AchievementType::CleanStreak,
            AchievementType::FirstBlood,
            AchievementType::LagBuster,
            AchievementType::KeyMaster,
            AchievementType::StatsFirstBlood,
            AchievementType::Champion,
        ]
    }

    /// Get all achievements required for Champion (all except Champion itself)
    pub fn champion_requirements() -> [AchievementType; 11] {
        [
            AchievementType::Connected,
            AchievementType::FirstLoad,
            AchievementType::Scaled,
            AchievementType::StatsDone,
            AchievementType::PartitionExplorer,
            AchievementType::HighThroughput,
            AchievementType::CleanStreak,
            AchievementType::FirstBlood,
            AchievementType::LagBuster,
            AchievementType::KeyMaster,
            AchievementType::StatsFirstBlood,
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
        assert!(!AchievementType::ParseError.is_step());
        assert!(!AchievementType::MissingFields.is_step());
        assert!(!AchievementType::PartitionExplorer.is_step());
    }

    #[test]
    fn test_is_error() {
        assert!(AchievementType::ParseError.is_error());
        assert!(AchievementType::MissingFields.is_error());
        assert!(!AchievementType::Connected.is_error());
        assert!(!AchievementType::PartitionExplorer.is_error());
    }

    #[test]
    fn test_all_steps_count() {
        assert_eq!(AchievementType::all_steps().len(), 4);
    }

    #[test]
    fn test_all_bonus_count() {
        assert_eq!(AchievementType::all_bonus().len(), 8);
    }

    #[test]
    fn test_champion_requirements() {
        let reqs = AchievementType::champion_requirements();
        assert_eq!(reqs.len(), 11);
        assert!(!reqs.contains(&AchievementType::Champion));
    }
}
