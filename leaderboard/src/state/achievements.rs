use serde::{Deserialize, Serialize};
use std::fmt;

/// Achievement types - minimal set for tracking tutorial progress
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AchievementType {
    // Step-based progress achievements
    Connected,     // Step 1: Consumer group active
    FirstLoad,     // Step 3: First message to actions topic
    Scaled,        // Step 4: 2+ consumers in group
    WatchlistDone, // Step 5: Message to watchlist topic

    // Error achievements (educational feedback)
    ParseError,    // Invalid JSON
    MissingFields, // Missing required fields

    // Bonus achievements
    PartitionExplorer, // 3+ consumers (teaches partition limits parallelism)
    HighThroughput,    // 100+ valid actions
    CleanStreak,       // 50+ messages with 0 errors
    FirstBlood,        // First team to get Connected
    LagBuster,         // Build up 100+ lag, then consume it all
    Champion,          // First team to complete ALL achievements
}

impl AchievementType {
    /// Get the emoji for this achievement
    pub fn emoji(&self) -> &'static str {
        match self {
            AchievementType::Connected => "1️⃣",
            AchievementType::FirstLoad => "3️⃣",
            AchievementType::Scaled => "4️⃣",
            AchievementType::WatchlistDone => "5️⃣",
            AchievementType::ParseError => "❌",
            AchievementType::MissingFields => "❓",
            AchievementType::PartitionExplorer => "🔬",
            AchievementType::HighThroughput => "📈",
            AchievementType::CleanStreak => "✨",
            AchievementType::FirstBlood => "⚔️",
            AchievementType::LagBuster => "🚀",
            AchievementType::Champion => "🏆",
        }
    }

    /// Get the name of this achievement
    pub fn name(&self) -> &'static str {
        match self {
            AchievementType::Connected => "Connected",
            AchievementType::FirstLoad => "First Load",
            AchievementType::Scaled => "Scaled",
            AchievementType::WatchlistDone => "Watchlist",
            AchievementType::ParseError => "Parse Error",
            AchievementType::MissingFields => "Missing Fields",
            AchievementType::PartitionExplorer => "Partition Explorer",
            AchievementType::HighThroughput => "High Throughput",
            AchievementType::CleanStreak => "Clean Streak",
            AchievementType::FirstBlood => "First Blood",
            AchievementType::LagBuster => "Lag Buster",
            AchievementType::Champion => "Champion",
        }
    }

    /// Check if this is a step achievement (progress)
    pub fn is_step(&self) -> bool {
        matches!(
            self,
            AchievementType::Connected
                | AchievementType::FirstLoad
                | AchievementType::Scaled
                | AchievementType::WatchlistDone
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
            AchievementType::WatchlistDone,
        ]
    }

    /// Get all error achievements
    pub fn all_errors() -> [AchievementType; 2] {
        [AchievementType::ParseError, AchievementType::MissingFields]
    }

    /// Get all bonus achievements in display order
    pub fn all_bonus() -> [AchievementType; 6] {
        [
            AchievementType::PartitionExplorer,
            AchievementType::HighThroughput,
            AchievementType::CleanStreak,
            AchievementType::FirstBlood,
            AchievementType::LagBuster,
            AchievementType::Champion,
        ]
    }

    /// Get all achievements required for Champion (all except Champion itself)
    pub fn champion_requirements() -> [AchievementType; 9] {
        [
            AchievementType::Connected,
            AchievementType::FirstLoad,
            AchievementType::Scaled,
            AchievementType::WatchlistDone,
            AchievementType::PartitionExplorer,
            AchievementType::HighThroughput,
            AchievementType::CleanStreak,
            AchievementType::FirstBlood,
            AchievementType::LagBuster,
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
        assert!(AchievementType::WatchlistDone.is_step());
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
    fn test_all_errors_count() {
        assert_eq!(AchievementType::all_errors().len(), 2);
    }

    #[test]
    fn test_all_bonus_count() {
        assert_eq!(AchievementType::all_bonus().len(), 6);
    }

    #[test]
    fn test_champion_requirements() {
        let reqs = AchievementType::champion_requirements();
        assert_eq!(reqs.len(), 9);
        assert!(!reqs.contains(&AchievementType::Champion));
    }
}
