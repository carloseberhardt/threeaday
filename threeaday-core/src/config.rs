use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub reminder_interval_minutes: u64,
    pub daily_reset_time: String,
    pub max_reminders_per_day: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            reminder_interval_minutes: 45,
            daily_reset_time: "06:00".to_string(),
            max_reminders_per_day: 8,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;
        
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read config from {}", config_path.display()))?;
            
            let config: Config = toml::from_str(&content)
                .with_context(|| format!("Failed to parse config from {}", config_path.display()))?;
            
            Ok(config)
        } else {
            // Create default config
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }
    
    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
        }
        
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config to TOML")?;
        
        fs::write(&config_path, content)
            .with_context(|| format!("Failed to write config to {}", config_path.display()))?;
        
        Ok(())
    }
    
    pub fn get_config_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("", "", "threeaday")
            .context("Failed to get project directories")?;
        Ok(proj_dirs.config_dir().join("config.toml"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::TempDir;

    fn setup_test_env() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        env::set_var("XDG_CONFIG_HOME", temp_dir.path());
        temp_dir
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.reminder_interval_minutes, 45);
        assert_eq!(config.daily_reset_time, "06:00");
        assert_eq!(config.max_reminders_per_day, 8);
    }

    #[test]
    fn test_config_load_creates_default_when_missing() {
        let _temp_dir = setup_test_env();
        
        // Config file shouldn't exist yet
        let config_path = Config::get_config_path().unwrap();
        assert!(!config_path.exists());
        
        // Loading should create default config
        let config = Config::load().unwrap();
        assert_eq!(config.reminder_interval_minutes, 45);
        assert_eq!(config.daily_reset_time, "06:00");
        assert_eq!(config.max_reminders_per_day, 8);
        
        // Config file should now exist
        assert!(config_path.exists());
    }

    #[test]
    fn test_config_load_reads_existing_file() {
        let _temp_dir = setup_test_env();
        
        // Create a custom config
        let custom_config = Config {
            reminder_interval_minutes: 30,
            daily_reset_time: "07:30".to_string(),
            max_reminders_per_day: 5,
        };
        custom_config.save().unwrap();
        
        // Loading should read the custom values
        let loaded_config = Config::load().unwrap();
        assert_eq!(loaded_config.reminder_interval_minutes, 30);
        assert_eq!(loaded_config.daily_reset_time, "07:30");
        assert_eq!(loaded_config.max_reminders_per_day, 5);
    }

    #[test]
    fn test_config_save_and_load_roundtrip() {
        let _temp_dir = setup_test_env();
        
        let original_config = Config {
            reminder_interval_minutes: 120,
            daily_reset_time: "05:00".to_string(),
            max_reminders_per_day: 10,
        };
        
        // Save and reload
        original_config.save().unwrap();
        let loaded_config = Config::load().unwrap();
        
        assert_eq!(loaded_config.reminder_interval_minutes, 120);
        assert_eq!(loaded_config.daily_reset_time, "05:00");
        assert_eq!(loaded_config.max_reminders_per_day, 10);
    }

    #[test]
    fn test_config_save_creates_directory() {
        let _temp_dir = setup_test_env();
        
        let config_path = Config::get_config_path().unwrap();
        let parent_dir = config_path.parent().unwrap();
        
        // Parent directory shouldn't exist initially
        assert!(!parent_dir.exists());
        
        // Saving should create the directory
        let config = Config::default();
        config.save().unwrap();
        
        assert!(parent_dir.exists());
        assert!(config_path.exists());
    }

    #[test]
    fn test_config_load_handles_invalid_toml() {
        let _temp_dir = setup_test_env();
        
        // Create invalid TOML file
        let config_path = Config::get_config_path().unwrap();
        fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        fs::write(&config_path, "invalid toml content [[[").unwrap();
        
        // Loading should fail gracefully
        let result = Config::load();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to parse config"));
    }

    #[test]
    fn test_config_get_config_path() {
        let _temp_dir = setup_test_env();
        
        let path = Config::get_config_path().unwrap();
        assert!(path.to_string_lossy().contains("threeaday"));
        assert!(path.to_string_lossy().ends_with("config.toml"));
    }

    #[test]
    fn test_config_partial_toml() {
        let _temp_dir = setup_test_env();
        
        // Create partial TOML with only some fields
        let config_path = Config::get_config_path().unwrap();
        fs::create_dir_all(config_path.parent().unwrap()).unwrap();
        fs::write(&config_path, r#"
reminder_interval_minutes = 60
# missing daily_reset_time and max_reminders_per_day
"#).unwrap();
        
        // Loading should handle missing fields with defaults
        let result = Config::load();
        // This should fail because TOML deserialization requires all fields
        // unless we add #[serde(default)] attributes
        assert!(result.is_err());
    }
}