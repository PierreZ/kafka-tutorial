use anyhow::Result;
use config::{Config, File};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub brokers: String,
    pub username: String,
    pub password: String,
    #[serde(default = "default_security_protocol")]
    pub security_protocol: String,
    #[serde(default = "default_sasl_mechanism")]
    pub sasl_mechanism: String,
    #[serde(default = "default_instances_per_team")]
    pub instances_per_team: u32,
}

fn default_security_protocol() -> String {
    "SASL_PLAINTEXT".to_string()
}

fn default_sasl_mechanism() -> String {
    "PLAIN".to_string()
}

fn default_instances_per_team() -> u32 {
    1
}

impl Settings {
    pub fn from_file(path: &str) -> Result<Self> {
        let config = Config::builder()
            .add_source(File::with_name(path))
            .build()?;
        Ok(config.try_deserialize()?)
    }
}
