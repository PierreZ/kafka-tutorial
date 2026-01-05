use crate::state::team::{ConsumerGroupStatus, GroupState};
use rdkafka::consumer::{BaseConsumer, Consumer};
use std::time::Duration;
use tracing::warn;

/// Map rdkafka group state string to our GroupState enum
fn map_group_state(state: &str) -> GroupState {
    match state.to_lowercase().as_str() {
        "stable" => GroupState::Active,
        "preparingrebalance" | "completingrebalance" => GroupState::Rebalancing,
        "empty" | "dead" => GroupState::Empty,
        _ => GroupState::Unknown,
    }
}

/// Fetch consumer group status for all teams using real Kafka group list
pub async fn fetch_consumer_group_statuses(
    brokers: &str,
    username: &str,
    password: &str,
    security_protocol: &str,
    sasl_mechanism: &str,
) -> Vec<ConsumerGroupStatus> {
    // Create a consumer to fetch group list
    let consumer: BaseConsumer =
        match kafka_common::kafka::new_sasl_config(brokers, username, password, security_protocol, sasl_mechanism)
            .set("group.id", "leaderboard-group-monitor")
            .create()
        {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to create consumer for group monitoring: {:?}", e);
                return fallback_empty_statuses();
            }
        };

    // Fetch all consumer groups
    match consumer.fetch_group_list(None, Duration::from_secs(5)) {
        Ok(group_list) => {
            let mut statuses: Vec<ConsumerGroupStatus> = group_list
                .groups()
                .iter()
                .filter(|g| g.name().starts_with("team-"))
                .map(|g| ConsumerGroupStatus {
                    team_name: g.name().to_string(),
                    state: map_group_state(g.state()),
                    members: g.members().len() as u32,
                    lag: 0, // Lag is calculated separately
                })
                .collect();

            // Ensure all teams are represented (add missing ones as Unknown)
            for i in 1..=crate::NUM_TEAMS {
                let team_name = format!("team-{}", i);
                if !statuses.iter().any(|s| s.team_name == team_name) {
                    statuses.push(ConsumerGroupStatus {
                        team_name,
                        state: GroupState::Unknown,
                        members: 0,
                        lag: 0,
                    });
                }
            }

            // Sort by team number for consistent ordering
            statuses.sort_by(|a, b| {
                let a_num: u32 = a
                    .team_name
                    .strip_prefix("team-")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                let b_num: u32 = b
                    .team_name
                    .strip_prefix("team-")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                a_num.cmp(&b_num)
            });

            statuses
        }
        Err(e) => {
            warn!("Failed to fetch group list: {:?}", e);
            fallback_empty_statuses()
        }
    }
}

/// Return empty statuses for all teams as fallback
fn fallback_empty_statuses() -> Vec<ConsumerGroupStatus> {
    (1..=crate::NUM_TEAMS)
        .map(|i| ConsumerGroupStatus {
            team_name: format!("team-{}", i),
            state: GroupState::Unknown,
            members: 0,
            lag: 0,
        })
        .collect()
}
