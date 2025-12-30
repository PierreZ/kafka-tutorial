use anyhow::Result;
use rdkafka::config::ClientConfig;
use rdkafka::producer::FutureProducer;

/// Create a Kafka producer with the standard configuration
pub fn create_producer(
    brokers: &str,
    username: &str,
    password: &str,
) -> Result<FutureProducer> {
    let producer: FutureProducer = ClientConfig::new()
        .set("client.id", "leaderboard-producer")
        .set("bootstrap.servers", brokers)
        .set("message.timeout.ms", "5000")
        .set("security.protocol", "SASL_SSL")
        .set("sasl.mechanisms", "PLAIN")
        .set("sasl.username", username)
        .set("sasl.password", password)
        .create()?;

    Ok(producer)
}
