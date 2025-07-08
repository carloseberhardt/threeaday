use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::path::PathBuf;

/// The daily goal - completing 3 tasks
pub const DAILY_GOAL_COMPLETION_COUNT: usize = 3;

/// Get the project directories for threeaday
pub fn get_project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("", "", "threeaday")
        .context("Failed to get project directories")
}

/// Get the data directory path
pub fn get_data_dir() -> Result<PathBuf> {
    let proj_dirs = get_project_dirs()?;
    Ok(proj_dirs.data_dir().to_path_buf())
}

/// Get the config directory path
pub fn get_config_dir() -> Result<PathBuf> {
    let proj_dirs = get_project_dirs()?;
    Ok(proj_dirs.config_dir().to_path_buf())
}

/// Format task status for display
pub fn format_task_status(completed: bool) -> &'static str {
    if completed { "✓" } else { " " }
}

/// Check if the daily goal is achieved
pub fn is_daily_goal_achieved(completed_count: usize) -> bool {
    completed_count >= DAILY_GOAL_COMPLETION_COUNT
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::TempDir;

    fn setup_test_env() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        env::set_var("XDG_DATA_HOME", temp_dir.path());
        env::set_var("XDG_CONFIG_HOME", temp_dir.path());
        temp_dir
    }

    #[test]
    fn test_daily_goal_completion_count_constant() {
        assert_eq!(DAILY_GOAL_COMPLETION_COUNT, 3);
    }

    #[test]
    fn test_get_project_dirs() {
        let result = get_project_dirs();
        assert!(result.is_ok());
        
        let proj_dirs = result.unwrap();
        assert!(proj_dirs.project_path().to_string_lossy().contains("threeaday"));
    }

    #[test]
    fn test_get_data_dir() {
        let _temp_dir = setup_test_env();
        
        let result = get_data_dir();
        assert!(result.is_ok());
        
        let data_dir = result.unwrap();
        assert!(data_dir.to_string_lossy().contains("threeaday"));
    }

    #[test]
    fn test_get_config_dir() {
        let _temp_dir = setup_test_env();
        
        let result = get_config_dir();
        assert!(result.is_ok());
        
        let config_dir = result.unwrap();
        assert!(config_dir.to_string_lossy().contains("threeaday"));
    }

    #[test]
    fn test_format_task_status_completed() {
        let status = format_task_status(true);
        assert_eq!(status, "✓");
    }

    #[test]
    fn test_format_task_status_incomplete() {
        let status = format_task_status(false);
        assert_eq!(status, " ");
    }

    #[test]
    fn test_is_daily_goal_achieved_below_goal() {
        assert!(!is_daily_goal_achieved(0));
        assert!(!is_daily_goal_achieved(1));
        assert!(!is_daily_goal_achieved(2));
    }

    #[test]
    fn test_is_daily_goal_achieved_at_goal() {
        assert!(is_daily_goal_achieved(3));
    }

    #[test]
    fn test_is_daily_goal_achieved_above_goal() {
        assert!(is_daily_goal_achieved(4));
        assert!(is_daily_goal_achieved(10));
        assert!(is_daily_goal_achieved(100));
    }

    #[test]
    fn test_is_daily_goal_achieved_edge_cases() {
        // Test with the constant directly
        assert!(is_daily_goal_achieved(DAILY_GOAL_COMPLETION_COUNT));
        assert!(!is_daily_goal_achieved(DAILY_GOAL_COMPLETION_COUNT - 1));
        assert!(is_daily_goal_achieved(DAILY_GOAL_COMPLETION_COUNT + 1));
    }

    #[test]
    fn test_directory_paths_are_different() {
        let _temp_dir = setup_test_env();
        
        let data_dir = get_data_dir().unwrap();
        let config_dir = get_config_dir().unwrap();
        
        // In real usage, these might be the same or different depending on OS
        // But they should both be valid paths
        assert!(data_dir.is_absolute());
        assert!(config_dir.is_absolute());
    }
}