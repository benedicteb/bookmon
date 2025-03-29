#[cfg(test)]
mod tests {
    use bookmon::config::Settings;

    #[test]
    fn test_load_config() {
        let settings = Settings::load().expect("Failed to load config");
        
        // Test app settings
        assert_eq!(settings.app_name, "BookMon");
        assert_eq!(settings.debug, false);
        
        // Test database settings
        assert_eq!(settings.database.host, "localhost");
        assert_eq!(settings.database.port, 5432);
        assert_eq!(settings.database.name, "bookmon_db");
        
        // Test feature flags
        assert_eq!(settings.features.enable_logging, true);
        assert_eq!(settings.features.enable_metrics, false);
    }
} 