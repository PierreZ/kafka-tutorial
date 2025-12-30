use anyhow::Result;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::StreamConsumer;

/// Create a Kafka consumer with the standard configuration
pub fn create_consumer(
    brokers: &str,
    username: &str,
    password: &str,
    group_id: &str,
) -> Result<StreamConsumer> {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", group_id)
        .set("bootstrap.servers", brokers)
        .set("enable.auto.commit", "true")
        .set("auto.commit.interval.ms", "5000")
        .set("auto.offset.reset", "latest")
        .set("security.protocol", "SASL_SSL")
        .set("sasl.mechanisms", "PLAIN")
        .set("sasl.username", username)
        .set("sasl.password", password)
        .set("session.timeout.ms", "10000")
        .create()?;

    Ok(consumer)
}

/// Create a consumer that reads from the beginning (for restoring state)
pub fn create_restore_consumer(
    brokers: &str,
    username: &str,
    password: &str,
    group_id: &str,
) -> Result<StreamConsumer> {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", group_id)
        .set("bootstrap.servers", brokers)
        .set("enable.auto.commit", "false")
        .set("auto.offset.reset", "earliest")
        .set("security.protocol", "SASL_SSL")
        .set("sasl.mechanisms", "PLAIN")
        .set("sasl.username", username)
        .set("sasl.password", password)
        .create()?;

    Ok(consumer)
}
