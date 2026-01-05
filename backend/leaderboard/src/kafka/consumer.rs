use anyhow::{Context, Result};
use rdkafka::consumer::{BaseConsumer, Consumer, StreamConsumer};
use std::time::Duration;

/// Create a Kafka consumer with the standard configuration
pub fn create_consumer(
    brokers: &str,
    username: &str,
    password: &str,
    group_id: &str,
    security_protocol: &str,
    sasl_mechanism: &str,
) -> Result<StreamConsumer> {
    let consumer: StreamConsumer =
        kafka_common::kafka::new_sasl_config(brokers, username, password, security_protocol, sasl_mechanism)
            .set("group.id", group_id)
            .set("enable.auto.commit", "true")
            .set("auto.commit.interval.ms", "5000")
            .set("auto.offset.reset", "latest")
            .set("session.timeout.ms", "10000")
            .create()?;

    Ok(consumer)
}

/// Test Kafka connection by fetching broker metadata.
/// Returns an error if connection fails, causing the app to exit.
pub fn test_connection(
    brokers: &str,
    username: &str,
    password: &str,
    security_protocol: &str,
    sasl_mechanism: &str,
) -> Result<()> {
    let consumer: BaseConsumer = kafka_common::kafka::new_sasl_config(
        brokers,
        username,
        password,
        security_protocol,
        sasl_mechanism,
    )
    .set("group.id", "leaderboard-connection-test")
    .create()
    .context("Failed to create Kafka consumer")?;

    // Try to fetch metadata with a short timeout
    consumer
        .fetch_metadata(None, Duration::from_secs(10))
        .context(format!(
            "Failed to connect to Kafka broker at {}. Please check your connection settings.",
            brokers
        ))?;

    Ok(())
}
