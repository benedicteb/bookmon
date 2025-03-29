#[cfg(test)]
mod tests {
    use bookmon::config::Settings;

    #[test]
    fn test_load_config() {
        let settings = Settings::load().expect("Failed to load config");
        
        // Test app settings
        assert_eq!(settings.app_name, "BookMon");
        assert_eq!(settings.debug, false);

        // Test storage settings
        assert_eq!(settings.storage_file, "./books.json");
    }
} 
