use serde::Deserialize;
use config::{Config, ConfigError, File};

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct FeatureFlags {
    pub enable_logging: bool,
    pub enable_metrics: bool,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub app_name: String,
    pub debug: bool,
    pub database: DatabaseConfig,
    pub features: FeatureFlags,
}

impl Settings {
    pub fn load() -> Result<Self, ConfigError> {
        let config = Config::builder()
            .add_source(File::with_name("config/default"))
            .build()?;

        config.try_deserialize()
    }
} 