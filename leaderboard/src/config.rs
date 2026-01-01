use anyhow::Result;
use config::{Config, File};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Settings {
    pub kafka: KafkaSettings,
    pub topics: TopicSettings,
    pub scoring: ScoringSettings,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KafkaSettings {
    pub brokers: String,
    pub username: String,
    pub password: String,
    pub consumer_group: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct TopicSettings {
    pub new_users: String,
    pub actions: String,
    pub watchlist: String,
    pub scorer_state: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct ScoringSettings {
    pub points_per_correct: u32,
    pub user_cache_max_size: usize,
}

impl Settings {
    pub fn from(path: &str) -> Result<Self> {
        let config = Config::builder()
            .add_source(File::with_name(path))
            .build()?;

        Ok(config.try_deserialize()?)
    }
}

impl Default for ScoringSettings {
    fn default() -> Self {
        Self {
            points_per_correct: 10,
            user_cache_max_size: 10000,
        }
    }
}
