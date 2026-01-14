use crate::filters::TeamFilter;
use crate::stats::StatsState;
use anyhow::Result;
use futures::StreamExt;
use kafka_common::topics::{ACTIONS_TOPIC, NEW_USERS_TOPIC, TEAM_STATS_TOPIC};
use kafka_common::{Action, TeamStats, User};
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::Message;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, trace};

pub struct TeamSimulator {
    team_number: u32,
    instance_id: u32,
    consumer: StreamConsumer,
    producer: Arc<FutureProducer>,
    filter: Box<dyn TeamFilter>,
    stats: Arc<StatsState>,
}

impl TeamSimulator {
    pub fn new(
        team_number: u32,
        instance_id: u32,
        consumer: StreamConsumer,
        producer: Arc<FutureProducer>,
        filter: Box<dyn TeamFilter>,
        stats: Arc<StatsState>,
    ) -> Self {
        Self {
            team_number,
            instance_id,
            consumer,
            producer,
            filter,
            stats,
        }
    }

    pub async fn run(self) -> Result<()> {
        let team_name = format!("team-{}", self.team_number);
        info!(
            "Starting {} instance {} - subscribing to {}",
            team_name, self.instance_id, NEW_USERS_TOPIC
        );

        self.consumer.subscribe(&[NEW_USERS_TOPIC])?;

        let mut stream = self.consumer.stream();

        while let Some(result) = stream.next().await {
            match result {
                Ok(message) => {
                    if let Some(payload) = message.payload() {
                        self.process_message(payload, &team_name).await;
                    }
                }
                Err(e) => {
                    error!(
                        "{} instance {}: Consumer error: {}",
                        team_name, self.instance_id, e
                    );
                }
            }
        }

        Ok(())
    }

    async fn process_message(&self, payload: &[u8], team_name: &str) {
        // Parse user
        let user: User = match serde_json::from_slice(payload) {
            Ok(u) => u,
            Err(e) => {
                debug!("Failed to parse user: {}", e);
                return;
            }
        };

        // Track every message processed
        self.stats.increment_processed();

        // Apply filter
        if let Some(filter_match) = self.filter.matches(&user) {
            // Produce Action message
            let action = Action::new(
                &user.email,
                filter_match.action_type,
                filter_match.reason,
                team_name,
            );

            let action_json = serde_json::to_string(&action).unwrap();
            self.produce(ACTIONS_TOPIC, &action_json, "").await;

            trace!("{}: Produced action for {}", team_name, user.email);

            // Produce stats update (key = team name for compaction)
            let (processed, flagged) = self.stats.increment_flagged();
            let stats = TeamStats::new(team_name, processed, flagged);
            let stats_json = serde_json::to_string(&stats).unwrap();
            self.produce(TEAM_STATS_TOPIC, &stats_json, team_name).await;

            trace!(
                "{}: Stats update (processed: {}, flagged: {})",
                team_name,
                processed,
                flagged
            );
        }
    }

    async fn produce(&self, topic: &str, value: &str, key: &str) {
        let record = FutureRecord::to(topic).payload(value).key(key);
        debug!(
            "team-{}: Producing to {} (key={:?}, len={})",
            self.team_number,
            topic,
            key,
            value.len()
        );
        if let Err((e, _)) = self.producer.send(record, Duration::from_secs(0)).await {
            error!("Failed to produce to {}: {}", topic, e);
        }
    }
}
