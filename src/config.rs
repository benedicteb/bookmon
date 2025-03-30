use serde::Deserialize;
use config::{Config, ConfigError, File, FileFormat};
use std::fs;
use dirs::config_dir;
use serde_yaml;
use std::path::PathBuf;

// Embed the default configuration directly in the binary
const DEFAULT_CONFIG: &str = include_str!("../config/default.yml");

/// Get the path to the config directory
fn get_config_dir() -> Result<PathBuf, ConfigError> {
    let config_dir = config_dir()
        .ok_or_else(|| ConfigError::Message("Could not find config directory".into()))?;
    
    Ok(config_dir.join("bookmon"))
}

/// Get the path to the config file
pub fn get_config_path() -> Result<PathBuf, ConfigError> {
    Ok(get_config_dir()?.join("config.yml"))
}

/// Create the config directory and file if they don't exist
fn create_config() -> Result<(), ConfigError> {
    let bookmon_dir = get_config_dir()?;
    fs::create_dir_all(&bookmon_dir)
        .map_err(|e| ConfigError::Message(format!("Failed to create config directory: {}", e)))?;
    
    let config_path = bookmon_dir.join("config.yml");
    if !config_path.exists() {
        fs::write(&config_path, "")
            .map_err(|e| ConfigError::Message(format!("Failed to create config file: {}", e)))?;
    }
    
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub app_name: String,
    pub debug: bool,
    #[serde(skip)]
    pub storage_file: String,
}

impl Settings {
    pub fn load() -> Result<Self, ConfigError> {
        // Create config directory and file if they don't exist
        create_config()?;
        
        // Get config path
        let config_path = get_config_path()?;

        // Create config builder
        let mut builder = Config::builder();

        // Add embedded default config
        builder = builder.add_source(File::from_str(DEFAULT_CONFIG, FileFormat::Yaml));

        // If user config exists, add it to override defaults
        if config_path.exists() {
            builder = builder.add_source(File::from(config_path.clone()));
        }

        // Build and deserialize config
        let mut settings: Settings = builder.build()?.try_deserialize()?;
        
        // Load storage file path from user config if it exists
        if config_path.exists() {
            if let Ok(contents) = fs::read_to_string(&config_path) {
                if let Ok(user_config) = serde_yaml::from_str::<serde_yaml::Value>(&contents) {
                    if let Some(storage_file) = user_config.get("storage_file").and_then(|v| v.as_str()) {
                        settings.storage_file = storage_file.to_string();
                    }
                }
            }
        }
        
        Ok(settings)
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let config_path = get_config_path()?;

        // Create a map of values to save
        let mut config_map = serde_yaml::Mapping::new();
        config_map.insert("storage_file".into(), self.storage_file.clone().into());

        // Write to file
        fs::write(&config_path, serde_yaml::to_string(&config_map)
            .map_err(|e| ConfigError::Message(format!("Failed to serialize config: {}", e)))?)
            .map_err(|e| ConfigError::Message(format!("Failed to save config file: {}", e)))?;

        Ok(())
    }
} 
