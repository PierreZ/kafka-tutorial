use anyhow::Result;
use config::{Config, File};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Settings {
    pub kafka: KafkaSettings,
    pub topics: TopicSettings,
    pub scoring: ScoringSettings,
    #[serde(default)]
    pub achievements: AchievementSettings,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KafkaSettings {
    pub brokers: String,
    pub username: String,
    pub password: String,
    pub consumer_group: String,
    #[serde(default = "default_security_protocol")]
    pub security_protocol: String,
    #[serde(default = "default_sasl_mechanism")]
    pub sasl_mechanism: String,
}

fn default_security_protocol() -> String {
    "SASL_PLAINTEXT".to_string()
}

fn default_sasl_mechanism() -> String {
    "PLAIN".to_string()
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
            user_cache_max_size: 10000,
        }
    }
}

/// Configurable thresholds for achievements
#[derive(Debug, Clone, Deserialize)]
pub struct AchievementSettings {
    /// Threshold for HighThroughput achievement (default: 100 valid actions)
    #[serde(default = "default_high_throughput")]
    pub high_throughput_threshold: u64,
    /// Threshold for CleanStreak achievement (default: 50 consecutive valid messages)
    #[serde(default = "default_clean_streak")]
    pub clean_streak_threshold: u64,
    /// Threshold for LagBuster achievement (default: 100 max lag before catching up)
    #[serde(default = "default_lag_buster")]
    pub lag_buster_threshold: i64,
    /// Member count for PartitionExplorer achievement (default: 3 consumers)
    #[serde(default = "default_partition_explorer")]
    pub partition_explorer_members: u32,
    /// Member count for Scaled achievement (default: 2 consumers)
    #[serde(default = "default_scaled")]
    pub scaled_members: u32,
}

fn default_high_throughput() -> u64 {
    100
}
fn default_clean_streak() -> u64 {
    50
}
fn default_lag_buster() -> i64 {
    100
}
fn default_partition_explorer() -> u32 {
    3
}
fn default_scaled() -> u32 {
    2
}

impl Default for AchievementSettings {
    fn default() -> Self {
        Self {
            high_throughput_threshold: default_high_throughput(),
            clean_streak_threshold: default_clean_streak(),
            lag_buster_threshold: default_lag_buster(),
            partition_explorer_members: default_partition_explorer(),
            scaled_members: default_scaled(),
        }
    }
}
