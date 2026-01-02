use anyhow::Result;
use rdkafka::consumer::StreamConsumer;

/// Create a Kafka consumer with the standard configuration
pub fn create_consumer(
    brokers: &str,
    username: &str,
    password: &str,
    group_id: &str,
) -> Result<StreamConsumer> {
    let consumer: StreamConsumer =
        kafka_common::kafka::new_sasl_ssl_config(brokers, username, password)
            .set("group.id", group_id)
            .set("enable.auto.commit", "true")
            .set("auto.commit.interval.ms", "5000")
            .set("auto.offset.reset", "latest")
            .set("session.timeout.ms", "10000")
            .create()?;

    Ok(consumer)
}
