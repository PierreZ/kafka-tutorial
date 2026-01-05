use crate::filters::TeamFilter;
use crate::watchlist::WatchlistState;
use anyhow::Result;
use futures::StreamExt;
use kafka_common::topics::{ACTIONS_TOPIC, NEW_USERS_TOPIC, WATCHLIST_TOPIC};
use kafka_common::{Action, User, WatchlistEntry};
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
    watchlist: Arc<WatchlistState>,
}

impl TeamSimulator {
    pub fn new(
        team_number: u32,
        instance_id: u32,
        consumer: StreamConsumer,
        producer: Arc<FutureProducer>,
        filter: Box<dyn TeamFilter>,
        watchlist: Arc<WatchlistState>,
    ) -> Self {
        Self {
            team_number,
            instance_id,
            consumer,
            producer,
            filter,
            watchlist,
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

            // Track for watchlist
            if let Some(flag_count) = self.watchlist.increment(&user.company_name) {
                let watchlist_entry =
                    WatchlistEntry::new(team_name, &user.company_name, flag_count);
                let watchlist_json = serde_json::to_string(&watchlist_entry).unwrap();
                self.produce(WATCHLIST_TOPIC, &watchlist_json, &user.company_name)
                    .await;

                debug!(
                    "{}: Watchlist alert for {} (count: {})",
                    team_name, user.company_name, flag_count
                );
            }
        }
    }

    async fn produce(&self, topic: &str, value: &str, key: &str) {
        let record = FutureRecord::to(topic).payload(value).key(key);
        debug!(
            "team-{}: Producing to {} (key={:?}, len={})",
            self.team_number, topic, key, value.len()
        );
        if let Err((e, _)) = self.producer.send(record, Duration::from_secs(0)).await {
            error!("Failed to produce to {}: {}", topic, e);
        }
    }
}
