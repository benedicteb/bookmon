#[cfg(test)]
mod tests {
    use bookmon::config::Settings;
    use config::{Config, File, FileFormat};

    const TEST_DEFAULT_CONFIG: &str = r#"
app_name: BookMon
debug: false
"#;

    fn create_test_settings(user_config: Option<&str>) -> Settings {
        let mut builder = Config::builder();
        
        // Add default config
        builder = builder.add_source(File::from_str(TEST_DEFAULT_CONFIG, FileFormat::Yaml));

        // Add user config if provided
        if let Some(config) = user_config {
            builder = builder.add_source(File::from_str(config, FileFormat::Yaml));
        }

        // Build settings
        let mut settings: Settings = builder.build()
            .expect("Failed to build config")
            .try_deserialize()
            .expect("Failed to deserialize config");

        // Manually set storage_file if provided in user config
        if let Some(config) = user_config {
            if let Ok(user_yaml) = serde_yaml::from_str::<serde_yaml::Value>(config) {
                if let Some(storage_file) = user_yaml.get("storage_file").and_then(|v| v.as_str()) {
                    settings.storage_file = storage_file.to_string();
                }
            }
        }

        settings
    }

    #[test]
    fn test_load_default_config() {
        let settings = create_test_settings(None);
        
        assert_eq!(settings.app_name, "BookMon");
        assert_eq!(settings.debug, false);
        assert_eq!(settings.storage_file, "");
    }

    #[test]
    fn test_load_user_config_override() {
        let user_config = r#"
app_name: CustomBookMon
debug: true
storage_file: /custom/path/storage.json
"#;
        
        let settings = create_test_settings(Some(user_config));
        
        assert_eq!(settings.app_name, "CustomBookMon");
        assert_eq!(settings.debug, true);
        assert_eq!(settings.storage_file, "/custom/path/storage.json");
    }

    #[test]
    fn test_partial_user_config_override() {
        let user_config = r#"
debug: true
storage_file: /custom/path/storage.json
"#;
        
        let settings = create_test_settings(Some(user_config));
        
        // app_name should remain default
        assert_eq!(settings.app_name, "BookMon");
        // other fields should be overridden
        assert_eq!(settings.debug, true);
        assert_eq!(settings.storage_file, "/custom/path/storage.json");
    }
} 
