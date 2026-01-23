use crate::state::team::{ConsumerGroupStatus, GroupState};
use rdkafka::consumer::{BaseConsumer, Consumer};
use rdkafka::TopicPartitionList;
use std::time::Duration;
use tracing::{debug, warn};

/// Timeout for Kafka admin operations (reduced for faster recovery)
const ADMIN_TIMEOUT: Duration = Duration::from_secs(2);

/// Map rdkafka group state string to our GroupState enum
fn map_group_state(state: &str) -> GroupState {
    match state.to_lowercase().as_str() {
        "stable" => GroupState::Active,
        "preparingrebalance" | "completingrebalance" => GroupState::Rebalancing,
        "empty" | "dead" => GroupState::Empty,
        _ => GroupState::Unknown,
    }
}

/// Create a BaseConsumer for admin operations
fn create_admin_consumer(
    brokers: &str,
    username: &str,
    password: &str,
    security_protocol: &str,
    sasl_mechanism: &str,
    group_id: &str,
) -> Result<BaseConsumer, rdkafka::error::KafkaError> {
    kafka_common::kafka::new_sasl_config(brokers, username, password, security_protocol, sasl_mechanism)
        .set("group.id", group_id)
        .create()
}

/// Fetch consumer group status for a single team (async-safe)
///
/// Wraps the blocking rdkafka calls in spawn_blocking to avoid blocking the async runtime.
pub async fn fetch_single_group_status_async(
    brokers: &str,
    username: &str,
    password: &str,
    security_protocol: &str,
    sasl_mechanism: &str,
    team_name: &str,
) -> ConsumerGroupStatus {
    let brokers = brokers.to_string();
    let username = username.to_string();
    let password = password.to_string();
    let security_protocol = security_protocol.to_string();
    let sasl_mechanism = sasl_mechanism.to_string();
    let team_name = team_name.to_string();

    let team_name_for_error = team_name.clone();

    tokio::task::spawn_blocking(move || {
        fetch_single_group_status_sync(
            &brokers,
            &username,
            &password,
            &security_protocol,
            &sasl_mechanism,
            &team_name,
        )
    })
    .await
    .unwrap_or_else(|e| {
        warn!("spawn_blocking failed for {}: {:?}", team_name_for_error, e);
        ConsumerGroupStatus {
            team_name: team_name_for_error,
            state: GroupState::Unknown,
            members: 0,
            lag: 0,
        }
    })
}

/// Fetch consumer group status for a single team (blocking)
fn fetch_single_group_status_sync(
    brokers: &str,
    username: &str,
    password: &str,
    security_protocol: &str,
    sasl_mechanism: &str,
    team_name: &str,
) -> ConsumerGroupStatus {
    let consumer: BaseConsumer = match create_admin_consumer(
        brokers,
        username,
        password,
        security_protocol,
        sasl_mechanism,
        "leaderboard-group-monitor",
    ) {
        Ok(c) => c,
        Err(e) => {
            debug!("Failed to create consumer for {}: {:?}", team_name, e);
            return ConsumerGroupStatus {
                team_name: team_name.to_string(),
                state: GroupState::Unknown,
                members: 0,
                lag: 0,
            };
        }
    };

    // Fetch group list filtered to this team
    match consumer.fetch_group_list(Some(team_name), ADMIN_TIMEOUT) {
        Ok(group_list) => {
            // Find this specific group
            if let Some(group) = group_list.groups().iter().find(|g| g.name() == team_name) {
                ConsumerGroupStatus {
                    team_name: team_name.to_string(),
                    state: map_group_state(group.state()),
                    members: group.members().len() as u32,
                    lag: 0, // Lag is calculated separately
                }
            } else {
                // Group doesn't exist yet
                ConsumerGroupStatus {
                    team_name: team_name.to_string(),
                    state: GroupState::Unknown,
                    members: 0,
                    lag: 0,
                }
            }
        }
        Err(e) => {
            debug!("Failed to fetch group {} status: {:?}", team_name, e);
            ConsumerGroupStatus {
                team_name: team_name.to_string(),
                state: GroupState::Unknown,
                members: 0,
                lag: 0,
            }
        }
    }
}

/// Fetch consumer group lag for a single team (async-safe)
///
/// Wraps the blocking rdkafka calls in spawn_blocking to avoid blocking the async runtime.
pub async fn fetch_consumer_group_lag_async(
    brokers: &str,
    username: &str,
    password: &str,
    security_protocol: &str,
    sasl_mechanism: &str,
    group_id: &str,
    topic: &str,
) -> Option<i64> {
    let brokers = brokers.to_string();
    let username = username.to_string();
    let password = password.to_string();
    let security_protocol = security_protocol.to_string();
    let sasl_mechanism = sasl_mechanism.to_string();
    let group_id = group_id.to_string();
    let topic = topic.to_string();

    let group_id_for_error = group_id.clone();

    tokio::task::spawn_blocking(move || {
        fetch_consumer_group_lag_sync(
            &brokers,
            &username,
            &password,
            &security_protocol,
            &sasl_mechanism,
            &group_id,
            &topic,
        )
    })
    .await
    .unwrap_or_else(|e| {
        warn!("spawn_blocking failed for lag check {}: {:?}", group_id_for_error, e);
        None
    })
}

/// Fetch committed offsets for a consumer group and calculate lag (blocking)
/// Returns total lag across all partitions, or None if failed
fn fetch_consumer_group_lag_sync(
    brokers: &str,
    username: &str,
    password: &str,
    security_protocol: &str,
    sasl_mechanism: &str,
    group_id: &str,
    topic: &str,
) -> Option<i64> {
    // Create a consumer with the target group to fetch committed offsets
    let consumer: BaseConsumer = match create_admin_consumer(
        brokers,
        username,
        password,
        security_protocol,
        sasl_mechanism,
        group_id,
    ) {
        Ok(c) => c,
        Err(e) => {
            debug!("Failed to create consumer for lag check {}: {:?}", group_id, e);
            return None;
        }
    };

    // Get topic metadata to find partitions
    let metadata = match consumer.fetch_metadata(Some(topic), ADMIN_TIMEOUT) {
        Ok(m) => m,
        Err(e) => {
            debug!("Failed to fetch metadata for {}: {:?}", topic, e);
            return None;
        }
    };

    let partitions: Vec<i32> = metadata
        .topics()
        .iter()
        .find(|t| t.name() == topic)
        .map(|t| t.partitions().iter().map(|p| p.id()).collect())
        .unwrap_or_default();

    if partitions.is_empty() {
        return None;
    }

    // Build topic partition list
    let mut tpl = TopicPartitionList::new();
    for partition in &partitions {
        tpl.add_partition(topic, *partition);
    }

    // Get committed offsets for this consumer group
    let committed = match consumer.committed_offsets(tpl, ADMIN_TIMEOUT) {
        Ok(c) => c,
        Err(e) => {
            debug!("Failed to fetch committed offsets for {}: {:?}", group_id, e);
            return None;
        }
    };

    // Calculate total lag
    let mut total_lag: i64 = 0;
    for partition in &partitions {
        // Get high watermark for this partition
        let (_, high) = match consumer.fetch_watermarks(topic, *partition, ADMIN_TIMEOUT) {
            Ok(w) => w,
            Err(_) => continue,
        };

        // Get committed offset for this partition
        let committed_offset = committed
            .find_partition(topic, *partition)
            .and_then(|tp| {
                let offset = tp.offset();
                match offset {
                    rdkafka::Offset::Offset(o) => Some(o),
                    _ => None,
                }
            })
            .unwrap_or(0);

        let partition_lag = (high - committed_offset).max(0);
        total_lag += partition_lag;
    }

    debug!("{}: lag={} for topic {}", group_id, total_lag, topic);
    Some(total_lag)
}
