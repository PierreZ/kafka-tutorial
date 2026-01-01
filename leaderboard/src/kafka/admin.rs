use crate::state::team::{ConsumerGroupStatus, GroupState};
use anyhow::Result;
#[allow(unused_imports)]
use rdkafka::admin::{AdminClient, AdminOptions};
use rdkafka::client::DefaultClientContext;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{BaseConsumer, Consumer};
use std::time::Duration;
use tracing::debug;

/// Create a Kafka admin client
pub fn create_admin_client(
    brokers: &str,
    username: &str,
    password: &str,
) -> Result<AdminClient<DefaultClientContext>> {
    let admin: AdminClient<DefaultClientContext> = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("security.protocol", "SASL_SSL")
        .set("sasl.mechanisms", "PLAIN")
        .set("sasl.username", username)
        .set("sasl.password", password)
        .create()?;

    Ok(admin)
}

/// Fetch consumer group status for all teams (team-1 through team-15)
/// Note: This is a simplified implementation that just initializes empty statuses.
/// Full consumer group monitoring would require using list_groups which is more complex.
pub async fn fetch_consumer_group_statuses(
    _admin: &AdminClient<DefaultClientContext>,
) -> Vec<ConsumerGroupStatus> {
    // For now, return empty status for each team
    // In a production implementation, you would use the admin client to list and describe groups
    let mut statuses = Vec::new();

    for i in 1..=15 {
        let team_name = format!("team-{}", i);
        statuses.push(ConsumerGroupStatus {
            team_name,
            state: GroupState::Unknown,
            members: 0,
            lag: 0,
        });
    }

    statuses
}

/// Get the high watermark (latest offset) for a topic
/// This is used to estimate consumer lag
pub fn fetch_topic_high_watermark(
    brokers: &str,
    username: &str,
    password: &str,
    topic: &str,
) -> Result<i64> {
    let consumer: BaseConsumer = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .set("security.protocol", "SASL_SSL")
        .set("sasl.mechanisms", "PLAIN")
        .set("sasl.username", username)
        .set("sasl.password", password)
        .set("group.id", "leaderboard-watermark-check")
        .create()?;

    // Get watermarks for partition 0 (assumes single partition or sums across partitions)
    // For multi-partition topics, you'd need to sum across all partitions
    let timeout = Duration::from_secs(5);
    let (low, high) = consumer.fetch_watermarks(topic, 0, timeout)?;
    debug!(
        "Topic {} partition 0: low={}, high={}",
        topic, low, high
    );

    Ok(high)
}

/// Calculate estimated lag for a team based on their action count vs topic high watermark
/// This is a simplified estimation - true lag would require querying committed offsets
/// for each team's consumer group, which rdkafka AdminClient doesn't directly support
pub fn estimate_team_lag(high_watermark: i64, team_action_count: u64) -> i64 {
    // Simple estimation: lag = messages in topic - messages processed by team
    // This assumes each new_user message should result in roughly one action (after filtering)
    // In reality, filtering reduces the number, so this is an upper bound estimate
    let estimated_position = team_action_count as i64;
    (high_watermark - estimated_position).max(0)
}
