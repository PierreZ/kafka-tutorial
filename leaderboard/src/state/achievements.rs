use serde::{Deserialize, Serialize};
use std::fmt;

/// Achievement types that teams can unlock
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AchievementType {
    // Progress achievements
    FirstSteps,    // First valid action
    Fifty,         // 50 valid actions
    Century,       // 100 valid actions
    Streak10,      // 10 consecutive correct

    // Mistake achievements (0 points, educational)
    ParseError,    // Invalid JSON
    GhostUser,     // Non-existent user
    Duplicate,     // Duplicate action
    MissingFields, // Missing required fields
    FalsePositive, // User doesn't match filter

    // Infrastructure achievements
    Connected,     // Consumer group active
    Scaled,        // 2+ consumers in group
}

impl AchievementType {
    /// Get the emoji badge for this achievement
    pub fn emoji(&self) -> &'static str {
        match self {
            AchievementType::FirstSteps => "🐣",
            AchievementType::Fifty => "🔥",
            AchievementType::Century => "💯",
            AchievementType::Streak10 => "⚡",
            AchievementType::ParseError => "❌",
            AchievementType::GhostUser => "👻",
            AchievementType::Duplicate => "2️⃣",
            AchievementType::MissingFields => "❓",
            AchievementType::FalsePositive => "🙈",
            AchievementType::Connected => "🔌",
            AchievementType::Scaled => "👥",
        }
    }

    /// Get the name of this achievement
    pub fn name(&self) -> &'static str {
        match self {
            AchievementType::FirstSteps => "First Steps",
            AchievementType::Fifty => "Fifty",
            AchievementType::Century => "Century",
            AchievementType::Streak10 => "Streak 10",
            AchievementType::ParseError => "Parse Error",
            AchievementType::GhostUser => "Ghost User",
            AchievementType::Duplicate => "Duplicate",
            AchievementType::MissingFields => "Missing Fields",
            AchievementType::FalsePositive => "False Positive",
            AchievementType::Connected => "Connected",
            AchievementType::Scaled => "Scaled",
        }
    }

    /// Get the points value for this achievement
    pub fn points(&self) -> u32 {
        match self {
            AchievementType::FirstSteps => 10,
            AchievementType::Fifty => 100,
            AchievementType::Century => 200,
            AchievementType::Streak10 => 50,
            // Mistake achievements are 0 points
            AchievementType::ParseError => 0,
            AchievementType::GhostUser => 0,
            AchievementType::Duplicate => 0,
            AchievementType::MissingFields => 0,
            AchievementType::FalsePositive => 0,
            // Infrastructure achievements
            AchievementType::Connected => 25,
            AchievementType::Scaled => 50,
        }
    }

    /// Check if this is a mistake achievement
    pub fn is_mistake(&self) -> bool {
        matches!(
            self,
            AchievementType::ParseError
                | AchievementType::GhostUser
                | AchievementType::Duplicate
                | AchievementType::MissingFields
                | AchievementType::FalsePositive
        )
    }
}

impl fmt::Display for AchievementType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.emoji(), self.name())
    }
}

/// An unlocked achievement with timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub achievement_type: AchievementType,
    pub unlocked_at: chrono::DateTime<chrono::Utc>,
}

impl Achievement {
    pub fn new(achievement_type: AchievementType) -> Self {
        Self {
            achievement_type,
            unlocked_at: chrono::Utc::now(),
        }
    }
}

/// Check for new progress achievements based on team state
pub fn check_progress_achievements(
    correct_count: u64,
    current_streak: u32,
    existing: &[Achievement],
) -> Vec<AchievementType> {
    let mut new_achievements = Vec::new();
    let existing_types: std::collections::HashSet<_> = existing
        .iter()
        .map(|a| a.achievement_type)
        .collect();

    // First Steps - first valid action
    if correct_count >= 1 && !existing_types.contains(&AchievementType::FirstSteps) {
        new_achievements.push(AchievementType::FirstSteps);
    }

    // Fifty - 50 valid actions
    if correct_count >= 50 && !existing_types.contains(&AchievementType::Fifty) {
        new_achievements.push(AchievementType::Fifty);
    }

    // Century - 100 valid actions
    if correct_count >= 100 && !existing_types.contains(&AchievementType::Century) {
        new_achievements.push(AchievementType::Century);
    }

    // Streak 10 - 10 consecutive correct
    if current_streak >= 10 && !existing_types.contains(&AchievementType::Streak10) {
        new_achievements.push(AchievementType::Streak10);
    }

    new_achievements
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_achievement_points() {
        assert_eq!(AchievementType::FirstSteps.points(), 10);
        assert_eq!(AchievementType::ParseError.points(), 0);
        assert_eq!(AchievementType::Connected.points(), 25);
    }

    #[test]
    fn test_is_mistake() {
        assert!(AchievementType::ParseError.is_mistake());
        assert!(!AchievementType::FirstSteps.is_mistake());
    }

    #[test]
    fn test_check_progress_achievements() {
        let existing = vec![];

        // First action should trigger FirstSteps
        let new = check_progress_achievements(1, 0, &existing);
        assert!(new.contains(&AchievementType::FirstSteps));

        // 50 actions should trigger Fifty
        let existing = vec![Achievement::new(AchievementType::FirstSteps)];
        let new = check_progress_achievements(50, 5, &existing);
        assert!(new.contains(&AchievementType::Fifty));
        assert!(!new.contains(&AchievementType::FirstSteps)); // Already unlocked
    }
}
