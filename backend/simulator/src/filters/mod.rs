mod rules;

use kafka_common::User;

/// Result of applying a filter to a user
pub struct FilterMatch {
    pub action_type: &'static str,
    pub reason: &'static str,
}

/// Trait for team-specific filtering logic
pub trait TeamFilter: Send + Sync {
    fn matches(&self, user: &User) -> Option<FilterMatch>;
}

/// Get the filter for a specific team number (1-15)
pub fn get_filter(team_number: u32) -> Box<dyn TeamFilter> {
    rules::get_filter(team_number)
}
