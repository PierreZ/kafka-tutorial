use config::{Config, ConfigError, File};
use kafka_common::User;
use rdkafka::producer::{FutureProducer, FutureRecord};
use serde::Deserialize;
use std::time::Duration;
use tokio::time;
use tracing::{error, info, trace};

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub interval_millis: u64,
    pub topic: String,
    pub brokers: String,
    pub username: String,
    pub password: String,
    #[serde(default = "default_security_protocol")]
    pub security_protocol: String,
    #[serde(default = "default_sasl_mechanism")]
    pub sasl_mechanism: String,
}

fn default_security_protocol() -> String {
    "SASL_SSL".to_string()
}

fn default_sasl_mechanism() -> String {
    "PLAIN".to_string()
}

// https://github.com/mehcode/config-rs/blob/master/examples/hierarchical-env/src/settings.rs#L39
impl Settings {
    pub fn from(path: &str) -> Result<Self, ConfigError> {
        let s = Config::builder()
            // Start off by merging in the "default" configuration file
            .add_source(File::with_name(path))
            .build()?;

        s.try_deserialize()
    }
}

pub async fn produce(settings: &Settings) {
    let producer: &FutureProducer = &kafka_common::kafka::new_sasl_config(
        &settings.brokers,
        &settings.username,
        &settings.password,
        &settings.security_protocol,
        &settings.sasl_mechanism,
    )
    .set("client.id", "kafka-demo-producer")
    .set("message.timeout.ms", "5000")
    .create()
    .expect("Producer creation error");

    info!(
        "Connected to {} (protocol: {}, mechanism: {}), producing to topic '{}'",
        settings.brokers, settings.security_protocol, settings.sasl_mechanism, settings.topic
    );

    let mut interval = time::interval(Duration::from_millis(settings.interval_millis));

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                info!("received a <CTRL-C>, exiting");
                return;
            }
            _ = interval.tick() => {
                send_message(producer, &settings.topic).await;
            }

        }
    }
}

async fn send_message(producer: &FutureProducer, topic_name: &str) {
    let user = User::random();
    let value = serde_json::to_string(&user).unwrap();
    match producer
        .send(
            FutureRecord::to(topic_name).payload(&value).key(""),
            Duration::from_secs(0),
        )
        .await
    {
        Ok(delivery) => trace!(
            "pushed message to partition {} at offset {}",
            delivery.partition,
            delivery.offset
        ),
        Err((error, message)) => error!("error pushing message: {:?}: {:?}", error, message),
    };
}
#[cfg(test)]
mod tests {
    use kafka_common::User;

    #[test]
    fn show_user() {
        dbg!(User::random());
    }
}
