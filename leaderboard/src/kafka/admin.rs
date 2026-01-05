use crate::state::team::{ConsumerGroupStatus, GroupState};
use anyhow::Result;
use rdkafka::consumer::{BaseConsumer, Consumer};
use std::time::Duration;
use tracing::{debug, warn};

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

/// Get the high watermark (latest offset) for a topic, summed across all partitions
pub fn fetch_topic_high_watermark(
    brokers: &str,
    username: &str,
    password: &str,
    security_protocol: &str,
    sasl_mechanism: &str,
    topic: &str,
) -> Result<i64> {
    let consumer: BaseConsumer =
        kafka_common::kafka::new_sasl_config(brokers, username, password, security_protocol, sasl_mechanism)
            .set("group.id", "leaderboard-watermark-check")
            .create()?;

    let timeout = Duration::from_secs(5);

    // Get topic metadata to find partition count
    let metadata = consumer.fetch_metadata(Some(topic), timeout)?;
    let partition_count = metadata
        .topics()
        .iter()
        .find(|t| t.name() == topic)
        .map(|t| t.partitions().len())
        .unwrap_or(1);

    // Sum watermarks across all partitions
    let mut total_high = 0i64;
    for partition in 0..partition_count as i32 {
        match consumer.fetch_watermarks(topic, partition, timeout) {
            Ok((_low, high)) => {
                debug!(
                    "Topic {} partition {}: low={}, high={}",
                    topic, partition, _low, high
                );
                total_high += high;
            }
            Err(e) => {
                warn!(
                    "Failed to fetch watermarks for {} partition {}: {:?}",
                    topic, partition, e
                );
            }
        }
    }

    debug!(
        "Topic {} total high watermark across {} partitions: {}",
        topic, partition_count, total_high
    );

    Ok(total_high)
}

/// Calculate estimated lag for a team based on messages consumed vs topic high watermark
pub fn estimate_team_lag(high_watermark: i64, messages_consumed: u64) -> i64 {
    // Lag = total messages in topic - messages consumed by team
    // This is accurate because messages_consumed tracks all messages the team processed
    (high_watermark - messages_consumed as i64).max(0)
}
