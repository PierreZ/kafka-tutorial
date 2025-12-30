use super::team::TeamState;
use anyhow::Result;
use futures::StreamExt;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// Create a Kafka producer for state persistence
pub fn create_state_producer(
    brokers: &str,
    username: &str,
    password: &str,
) -> Result<FutureProducer> {
    let producer: FutureProducer = ClientConfig::new()
        .set("client.id", "leaderboard-state-producer")
        .set("bootstrap.servers", brokers)
        .set("message.timeout.ms", "5000")
        .set("security.protocol", "SASL_SSL")
        .set("sasl.mechanisms", "PLAIN")
        .set("sasl.username", username)
        .set("sasl.password", password)
        .create()?;
    Ok(producer)
}

/// Persist team state to Kafka topic
pub async fn persist_state(
    producer: &FutureProducer,
    topic: &str,
    state: &TeamState,
) -> Result<()> {
    let key = state.team_name.as_bytes();
    let value = serde_json::to_string(state)?;

    match producer
        .send(
            FutureRecord::to(topic).key(key).payload(&value),
            Duration::from_secs(5),
        )
        .await
    {
        Ok(delivery) => {
            debug!(
                "Persisted state for {} to partition {} offset {}",
                state.team_name, delivery.partition, delivery.offset
            );
            Ok(())
        }
        Err((err, _)) => {
            error!("Failed to persist state for {}: {:?}", state.team_name, err);
            Err(err.into())
        }
    }
}

/// Restore all team states from the scorer_state topic
pub async fn restore_states(
    brokers: &str,
    username: &str,
    password: &str,
    topic: &str,
) -> Result<HashMap<String, TeamState>> {
    info!("Restoring state from topic: {}", topic);

    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", "leaderboard-state-restore")
        .set("bootstrap.servers", brokers)
        .set("enable.auto.commit", "false")
        .set("auto.offset.reset", "earliest")
        .set("security.protocol", "SASL_SSL")
        .set("sasl.mechanisms", "PLAIN")
        .set("sasl.username", username)
        .set("sasl.password", password)
        .create()?;

    // Subscribe to the topic
    consumer.subscribe(&[topic])?;

    let mut states: HashMap<String, TeamState> = HashMap::new();
    let timeout = Duration::from_secs(5);
    let mut stream = consumer.stream();

    // Read messages with a timeout
    loop {
        match tokio::time::timeout(timeout, stream.next()).await {
            Ok(Some(Ok(message))) => {
                if let Some(payload) = message.payload() {
                    match serde_json::from_slice::<TeamState>(payload) {
                        Ok(state) => {
                            debug!("Restored state for team: {}", state.team_name);
                            states.insert(state.team_name.clone(), state);
                        }
                        Err(e) => {
                            warn!("Failed to deserialize state: {:?}", e);
                        }
                    }
                }
            }
            Ok(Some(Err(e))) => {
                warn!("Error polling for state: {:?}", e);
            }
            Ok(None) => {
                // Stream ended
                break;
            }
            Err(_) => {
                // Timeout - no more messages
                break;
            }
        }
    }

    info!("Restored {} team states", states.len());
    Ok(states)
}
