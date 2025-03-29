use serde::Deserialize;
use config::{Config, ConfigError, File, FileFormat};

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub app_name: String,
    pub debug: bool,
    pub storage_file: String
}

impl Settings {
    pub fn load() -> Result<Self, ConfigError> {
        let config = Config::builder()
            .add_source(File::new("config/default", FileFormat::Yaml))
            .build()?;

        config.try_deserialize()
    }
} 
