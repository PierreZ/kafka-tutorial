use anyhow::Result;
use rdkafka::producer::FutureProducer;

/// Create a Kafka producer with the standard configuration
pub fn create_producer(
    brokers: &str,
    username: &str,
    password: &str,
    security_protocol: &str,
    sasl_mechanism: &str,
) -> Result<FutureProducer> {
    let producer: FutureProducer =
        kafka_common::kafka::new_sasl_config(brokers, username, password, security_protocol, sasl_mechanism)
            .set("client.id", "leaderboard-producer")
            .set("message.timeout.ms", "5000")
            .create()?;

    Ok(producer)
}
