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
    pub team_stats: String,
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
    /// Member count for Scaled achievement (default: 2 consumers)
    #[serde(default = "default_scaled")]
    pub scaled_members: u32,
}

fn default_scaled() -> u32 {
    2
}

impl Default for AchievementSettings {
    fn default() -> Self {
        Self {
            scaled_members: default_scaled(),
        }
    }
}
