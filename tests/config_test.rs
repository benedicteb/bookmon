#[cfg(test)]
mod tests {
    use bookmon::config::Settings;
    use std::fs;
    use dirs::config_dir;

    #[test]
    fn test_load_config() {
        // Get the config directory
        let config_dir = config_dir().expect("Could not find config directory");
        let bookmon_dir = config_dir.join("bookmon");
        let config_path = bookmon_dir.join("config.yml");

        // Backup existing config if it exists
        let config_backup = if config_path.exists() {
            let contents = fs::read_to_string(&config_path).expect("Failed to read config file");
            Some(contents)
        } else {
            None
        };

        // Remove existing config if it exists
        if config_path.exists() {
            fs::remove_file(&config_path).expect("Failed to remove config file");
        }

        // Load settings and verify defaults
        let settings = Settings::load().expect("Failed to load config");
        
        // Test app settings
        assert_eq!(settings.app_name, "BookMon");
        assert_eq!(settings.debug, false);

        // Test storage settings
        assert_eq!(settings.storage_file, "");

        // Restore config if it was backed up
        if let Some(contents) = config_backup {
            fs::create_dir_all(&bookmon_dir).expect("Failed to create config directory");
            fs::write(&config_path, contents).expect("Failed to restore config file");
        }
    }
} 
