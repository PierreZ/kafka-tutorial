use rdkafka::ClientConfig;

/// Configure a Kafka client with SASL_SSL authentication.
/// This is the common configuration used by both producer and leaderboard.
pub fn configure_sasl_ssl<'a>(
    config: &'a mut ClientConfig,
    brokers: &str,
    username: &str,
    password: &str,
) -> &'a mut ClientConfig {
    config
        .set("bootstrap.servers", brokers)
        .set("security.protocol", "SASL_SSL")
        .set("sasl.mechanisms", "PLAIN")
        .set("sasl.username", username)
        .set("sasl.password", password)
}

/// Create a new ClientConfig with SASL_SSL authentication pre-configured.
pub fn new_sasl_ssl_config(brokers: &str, username: &str, password: &str) -> ClientConfig {
    let mut config = ClientConfig::new();
    configure_sasl_ssl(&mut config, brokers, username, password);
    config
}
