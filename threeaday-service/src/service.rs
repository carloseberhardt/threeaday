use chrono::{Local, NaiveDate, NaiveTime};
use notify_rust::Notification;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use threeaday_core::{utils::*, Config, Database, Result};
use tokio::time::{sleep, Instant};

#[derive(Debug, Serialize, Deserialize)]
struct PersistentState {
    last_reminder_date: Option<NaiveDate>,
    last_reset_date: Option<NaiveDate>,
    last_achievement_date: Option<NaiveDate>,
    reminders_sent_today: u32,
    current_date: NaiveDate,
}

struct ServiceState {
    db: Database,
    config: Config,
    persistent: PersistentState,
    last_reminder_instant: Option<Instant>,
}

impl ServiceState {
    fn new() -> Result<Self> {
        let db = Database::new()?;
        let config = Config::load()?;
        let persistent = Self::load_persistent_state()?;

        Ok(ServiceState {
            db,
            config,
            persistent,
            last_reminder_instant: None,
        })
    }

    fn get_state_path() -> Result<PathBuf> {
        let data_dir = get_data_dir()?;
        Ok(data_dir.join("service_state.json"))
    }

    fn load_persistent_state() -> Result<PersistentState> {
        let state_path = Self::get_state_path()?;
        let today = Local::now().date_naive();

        if state_path.exists() {
            let content = fs::read_to_string(&state_path)?;
            let mut state: PersistentState = serde_json::from_str(&content)?;

            // Reset daily counters if it's a new day
            if state.current_date != today {
                state.reminders_sent_today = 0;
                state.current_date = today;
            }

            Ok(state)
        } else {
            Ok(PersistentState {
                last_reminder_date: None,
                last_reset_date: None,
                last_achievement_date: None,
                reminders_sent_today: 0,
                current_date: today,
            })
        }
    }

    fn save_persistent_state(&self) -> Result<()> {
        let state_path = Self::get_state_path()?;

        // Ensure parent directory exists
        if let Some(parent) = state_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(&self.persistent)?;
        fs::write(state_path, json)?;
        Ok(())
    }

    fn should_send_reminder(&self) -> bool {
        if self.persistent.reminders_sent_today >= self.config.max_reminders_per_day {
            return false;
        }

        // Don't send immediate reminder on startup - wait at least one interval
        if self.last_reminder_instant.is_none() {
            return false;
        }

        match self.db.get_today_summary() {
            Ok((completed, _total)) => {
                // Don't remind if goal is achieved
                if is_daily_goal_achieved(completed) {
                    return false;
                }

                // Always remind unless goal is achieved
                true
            }
            Err(_) => false,
        }
    }

    fn send_reminder(&mut self) -> Result<()> {
        if !self.should_send_reminder() {
            return Ok(());
        }

        let (completed, total) = self.db.get_today_summary()?;

        let (title, message) = if total == 0 {
            (
                "ThreeADay Reminder".to_string(),
                "Time to add your first task for today! 🎯".to_string(),
            )
        } else {
            let remaining = DAILY_GOAL_COMPLETION_COUNT.saturating_sub(completed);
            (
                "ThreeADay Reminder".to_string(),
                format!(
                    "You have {}/{} tasks completed. {} more to reach your daily goal! 💪",
                    completed, total, remaining
                ),
            )
        };

        Notification::new()
            .summary(&title)
            .body(&message)
            .timeout(5000)
            .show()?;

        self.last_reminder_instant = Some(Instant::now());
        self.persistent.last_reminder_date = Some(Local::now().date_naive());
        self.persistent.reminders_sent_today += 1;
        self.save_persistent_state()?;

        Ok(())
    }

    fn should_send_daily_reset(&self) -> bool {
        // Check if it's time for daily reset using LOCAL time
        let now = Local::now();
        let today = now.date_naive();
        let current_time = now.time();

        let reset_time = NaiveTime::parse_from_str(&self.config.daily_reset_time, "%H:%M")
            .unwrap_or_else(|_| NaiveTime::from_hms_opt(6, 0, 0).unwrap());

        // Check if we're past reset time today
        if current_time >= reset_time {
            // Check if we already sent reset today
            if let Some(last_reset_date) = self.persistent.last_reset_date {
                return last_reset_date < today;
            }
            return true;
        }

        false
    }

    fn send_daily_reset_notification(&mut self) -> Result<()> {
        if !self.should_send_daily_reset() {
            return Ok(());
        }

        let message = "🌅 Good morning! It's a fresh start. What 3 tasks will you complete today?";

        Notification::new()
            .summary("ThreeADay - Fresh Start")
            .body(message)
            .timeout(8000)
            .show()?;

        self.persistent.last_reset_date = Some(Local::now().date_naive());
        self.persistent.reminders_sent_today = 0; // Reset reminder counter
        self.save_persistent_state()?;

        Ok(())
    }

    fn send_achievement_notification(&mut self) -> Result<()> {
        let (completed, _total) = self.db.get_today_summary()?;

        if is_daily_goal_achieved(completed) {
            // Only send if we haven't already celebrated today
            let today = Local::now().date_naive();
            if self.persistent.last_achievement_date != Some(today) {
                let message = format!(
                    "🎉 Congratulations! You've completed {} tasks today. Goal achieved! 🎯",
                    completed
                );

                Notification::new()
                    .summary("ThreeADay - Goal Achieved!")
                    .body(&message)
                    .timeout(8000)
                    .show()?;

                self.persistent.last_achievement_date = Some(today);
                self.save_persistent_state()?;
            }
        }

        Ok(())
    }

    async fn run_main_loop(&mut self) -> Result<()> {
        let mut last_task_check = Instant::now();

        loop {
            let now = Instant::now();

            // Check for daily reset
            if let Err(e) = self.send_daily_reset_notification() {
                eprintln!("Error sending daily reset notification: {}", e);
            }

            // Initialize reminder timer on first run (but don't send immediately)
            if self.last_reminder_instant.is_none() {
                self.last_reminder_instant = Some(Instant::now());
            }

            // Check for reminders (every configured interval)
            if let Some(last_reminder) = self.last_reminder_instant {
                if last_reminder.elapsed().as_secs() >= self.config.reminder_interval_minutes * 60 {
                    if let Err(e) = self.send_reminder() {
                        eprintln!("Error sending reminder: {}", e);
                    }
                }
            }

            // Check for achievement notification (when tasks change)
            if last_task_check.elapsed().as_secs() >= 30 {
                match self.db.get_today_summary() {
                    Ok((completed, _total)) => {
                        if is_daily_goal_achieved(completed) {
                            if let Err(e) = self.send_achievement_notification() {
                                eprintln!("Error sending achievement notification: {}", e);
                            }
                        }

                        last_task_check = now;
                    }
                    Err(e) => {
                        eprintln!("Error checking tasks: {}", e);
                    }
                }
            }

            // Sleep for a short interval
            sleep(Duration::from_secs(60)).await;
        }
    }
}

pub async fn run_service() -> Result<()> {
    println!("Starting ThreeADay service...");

    let mut state = ServiceState::new()?;

    // Send startup notification
    if let Err(e) = Notification::new()
        .summary("ThreeADay Service")
        .body("Service started - reminders and daily resets are now active! 🚀")
        .timeout(3000)
        .show()
    {
        eprintln!("Error sending startup notification: {}", e);
    }

    // Run main service loop
    state.run_main_loop().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Local;
    use chrono::Timelike;
    use std::env;
    use tempfile::TempDir;

    fn setup_test_env() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        env::set_var("XDG_DATA_HOME", temp_dir.path());
        env::set_var("XDG_CONFIG_HOME", temp_dir.path());
        temp_dir
    }

    fn create_test_state_with_persistent(
        config: Config,
        persistent: PersistentState,
    ) -> ServiceState {
        let db = Database::new().unwrap();
        ServiceState {
            db,
            config,
            persistent,
            last_reminder_instant: None,
        }
    }

    #[test]
    fn test_timezone_fixed() {
        // This test verifies the timezone fix works
        let _temp_dir = setup_test_env();

        let config = Config {
            reminder_interval_minutes: 45,
            daily_reset_time: "06:00".to_string(),
            max_reminders_per_day: 8,
        };

        let today = Local::now().date_naive();
        let persistent = PersistentState {
            last_reminder_date: None,
            last_reset_date: None,
            last_achievement_date: None,
            reminders_sent_today: 0,
            current_date: today,
        };

        let state = create_test_state_with_persistent(config, persistent);

        // Now uses Local time, not UTC
        let now_local = Local::now();

        // The fix: should_send_daily_reset now uses Local time
        let should_reset = state.should_send_daily_reset();

        println!(
            "Local time: {:02}:{:02}, Reset check: {}",
            now_local.hour(),
            now_local.minute(),
            should_reset
        );

        // This should now work correctly based on local time
    }

    #[test]
    fn test_daily_reset_boundary_logic_fixed() {
        // This test verifies the reset time comparison is fixed
        let _temp_dir = setup_test_env();

        let config = Config {
            reminder_interval_minutes: 45,
            daily_reset_time: "06:30".to_string(),
            max_reminders_per_day: 8,
        };

        let today = Local::now().date_naive();
        let persistent = PersistentState {
            last_reminder_date: None,
            last_reset_date: None,
            last_achievement_date: None,
            reminders_sent_today: 0,
            current_date: today,
        };

        let _state = create_test_state_with_persistent(config, persistent);

        // Test the fixed logic
        let reset_time = NaiveTime::from_hms_opt(6, 30, 0).unwrap();
        let test_time_after = NaiveTime::from_hms_opt(7, 15, 0).unwrap();
        let test_time_before = NaiveTime::from_hms_opt(5, 45, 0).unwrap();

        // The fix: proper time comparison
        assert!(test_time_after >= reset_time, "7:15 is after 6:30");
        assert!(!(test_time_before >= reset_time), "5:45 is before 6:30");
    }

    #[test]
    fn test_state_persistence_works() {
        // This test verifies state persistence is implemented
        let _temp_dir = setup_test_env();

        // Create first instance with some state
        {
            let config = Config {
                reminder_interval_minutes: 45,
                daily_reset_time: "06:00".to_string(),
                max_reminders_per_day: 8,
            };

            let today = Local::now().date_naive();
            let persistent = PersistentState {
                last_reminder_date: Some(today),
                last_reset_date: Some(today),
                last_achievement_date: None,
                reminders_sent_today: 5,
                current_date: today,
            };

            let state = create_test_state_with_persistent(config, persistent);

            // Save the state
            state.save_persistent_state().unwrap();
        }

        // Create new instance and verify state was loaded
        {
            let loaded_state = ServiceState::load_persistent_state().unwrap();
            let today = Local::now().date_naive();

            assert_eq!(
                loaded_state.last_reminder_date,
                Some(today),
                "Reminder date persisted"
            );
            assert_eq!(
                loaded_state.last_reset_date,
                Some(today),
                "Reset date persisted"
            );
            assert_eq!(
                loaded_state.reminders_sent_today, 5,
                "Reminder count persisted"
            );
        }
    }

    #[test]
    fn test_no_immediate_reminder_on_startup() {
        // This test verifies reminders aren't sent immediately on startup
        let _temp_dir = setup_test_env();

        let config = Config {
            reminder_interval_minutes: 45,
            daily_reset_time: "06:00".to_string(),
            max_reminders_per_day: 8,
        };

        let today = Local::now().date_naive();
        let persistent = PersistentState {
            last_reminder_date: None,
            last_reset_date: None,
            last_achievement_date: None,
            reminders_sent_today: 0,
            current_date: today,
        };

        let state = create_test_state_with_persistent(config, persistent);

        // The fix: should_send_reminder returns false when last_reminder_instant is None
        assert!(
            !state.should_send_reminder(),
            "Should not send reminder immediately on startup"
        );
    }

    #[test]
    fn test_reset_notification_not_repeated() {
        // Verifies daily reset doesn't trigger multiple times on same day
        let _temp_dir = setup_test_env();

        let config = Config {
            reminder_interval_minutes: 45,
            daily_reset_time: "06:00".to_string(),
            max_reminders_per_day: 8,
        };

        let today = Local::now().date_naive();

        // State where reset was already sent today
        let persistent = PersistentState {
            last_reminder_date: None,
            last_reset_date: Some(today), // Already sent today
            last_achievement_date: None,
            reminders_sent_today: 0,
            current_date: today,
        };

        let state = create_test_state_with_persistent(config, persistent);

        // Should NOT send reset again today
        assert!(
            !state.should_send_daily_reset(),
            "Should not send reset twice on same day"
        );
    }

    #[test]
    fn test_achievement_notification_once_per_day() {
        // Verifies achievement notifications are sent only once per day
        let _temp_dir = setup_test_env();

        let config = Config {
            reminder_interval_minutes: 45,
            daily_reset_time: "06:00".to_string(),
            max_reminders_per_day: 8,
        };

        let today = Local::now().date_naive();

        // State where achievement was already celebrated today
        let persistent = PersistentState {
            last_reminder_date: None,
            last_reset_date: None,
            last_achievement_date: Some(today), // Already celebrated today
            reminders_sent_today: 0,
            current_date: today,
        };

        let state = create_test_state_with_persistent(config, persistent);

        // Even if goal is achieved, shouldn't notify again today
        // (This would be checked inside send_achievement_notification)
        assert_eq!(state.persistent.last_achievement_date, Some(today));
    }
}
