use rdkafka::ClientConfig;

/// Configure a Kafka client with SASL authentication.
/// This is the common configuration used by both producer and leaderboard.
pub fn configure_sasl<'a>(
    config: &'a mut ClientConfig,
    brokers: &str,
    username: &str,
    password: &str,
    security_protocol: &str,
    sasl_mechanism: &str,
) -> &'a mut ClientConfig {
    config
        .set("bootstrap.servers", brokers)
        .set("security.protocol", security_protocol)
        .set("sasl.mechanisms", sasl_mechanism)
        .set("sasl.username", username)
        .set("sasl.password", password)
}

/// Create a new ClientConfig with SASL authentication pre-configured.
pub fn new_sasl_config(
    brokers: &str,
    username: &str,
    password: &str,
    security_protocol: &str,
    sasl_mechanism: &str,
) -> ClientConfig {
    let mut config = ClientConfig::new();
    configure_sasl(&mut config, brokers, username, password, security_protocol, sasl_mechanism);
    config
}

/// Configure a Kafka client with SASL_SSL authentication (legacy helper).
/// This is kept for backwards compatibility.
pub fn configure_sasl_ssl<'a>(
    config: &'a mut ClientConfig,
    brokers: &str,
    username: &str,
    password: &str,
) -> &'a mut ClientConfig {
    configure_sasl(config, brokers, username, password, "SASL_SSL", "PLAIN")
}

/// Create a new ClientConfig with SASL_SSL authentication pre-configured (legacy helper).
pub fn new_sasl_ssl_config(brokers: &str, username: &str, password: &str) -> ClientConfig {
    new_sasl_config(brokers, username, password, "SASL_SSL", "PLAIN")
}
