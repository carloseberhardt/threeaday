mod db;

use anyhow::Result;
use chrono::{Duration, Local, NaiveTime};
use db::Database;
use directories::ProjectDirs;
use notify_rust::{Notification, Urgency};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration as StdDuration;
use tokio::time::interval;

#[derive(Debug, Serialize, Deserialize)]
struct ServiceConfig {
    reminder_interval_minutes: u64,
    daily_reset_time: String, // Store as "HH:MM" format
    max_reminders_per_day: u32,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            reminder_interval_minutes: 45, // Remind every 45 minutes
            daily_reset_time: "06:00".to_string(), // 6 AM reset
            max_reminders_per_day: 8, // Max 8 reminders per day
        }
    }
}

impl ServiceConfig {
    fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;
        
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let config: ServiceConfig = toml::from_str(&content)?;
            Ok(config)
        } else {
            // Create default config file
            let default_config = Self::default();
            default_config.save()?;
            Ok(default_config)
        }
    }
    
    fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;
        
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let content = toml::to_string_pretty(self)?;
        fs::write(&config_path, content)?;
        
        println!("Config saved to: {:?}", config_path);
        Ok(())
    }
    
    fn get_config_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("", "", "threeaday")
            .ok_or_else(|| anyhow::anyhow!("Failed to get project directories"))?;
        Ok(proj_dirs.config_dir().join("config.toml"))
    }
    
    fn get_reset_time(&self) -> Result<NaiveTime> {
        NaiveTime::parse_from_str(&self.daily_reset_time, "%H:%M")
            .map_err(|e| anyhow::anyhow!("Invalid time format '{}': {}", self.daily_reset_time, e))
    }
}

struct ServiceState {
    config: ServiceConfig,
    db: Database,
    reminders_sent_today: u32,
    last_reminder_date: chrono::NaiveDate,
}

impl ServiceState {
    fn new() -> Result<Self> {
        let config = ServiceConfig::load()?;
        let db = Database::new()?;
        let today = Local::now().date_naive();
        
        Ok(Self {
            config,
            db,
            reminders_sent_today: 0,
            last_reminder_date: today,
        })
    }

    fn is_new_day(&self) -> bool {
        let today = Local::now().date_naive();
        today != self.last_reminder_date
    }

    fn reset_daily_counters(&mut self) {
        self.reminders_sent_today = 0;
        self.last_reminder_date = Local::now().date_naive();
    }

    fn should_send_reminder(&self) -> bool {
        if self.reminders_sent_today >= self.config.max_reminders_per_day {
            return false;
        }

        match self.db.get_today_summary() {
            Ok((completed, total)) => {
                // Don't remind if goal is achieved
                if completed >= 3 {
                    return false;
                }
                
                // Always remind unless goal is achieved
                true
            }
            Err(_) => false,
        }
    }

    fn send_reminder(&mut self) -> Result<()> {
        let (completed, total) = self.db.get_today_summary()?;
        
        let message = if total == 0 {
            "💡 Add your first task for today!".to_string()
        } else if completed == 0 {
            format!("🚀 Time to tackle your {} tasks!", total)
        } else {
            format!("🏃 Progress: {}/{} tasks completed. Keep going!", completed, total)
        };

        // Play notification sound
        self.play_notification_sound();

        Notification::new()
            .summary("ThreeADay")
            .body(&message)
            .urgency(Urgency::Critical)
            .timeout(0) // Persistent notification
            .show()
            .map_err(|e| anyhow::anyhow!("Failed to send notification: {}", e))?;

        self.reminders_sent_today += 1;
        println!("Sent persistent reminder: {}", message);
        
        Ok(())
    }

    fn check_for_achievements(&self) -> Result<()> {
        let (completed, _) = self.db.get_today_summary()?;
        
        if completed >= 3 {
            let message = format!("🎯 Amazing! You've completed {} tasks today! Great momentum!", completed);
            
            // Play achievement sound
            self.play_notification_sound();
            
            Notification::new()
                .summary("ThreeADay - Goal Achieved!")
                .body(&message)
                .urgency(Urgency::Critical)
                .timeout(0) // Persistent notification for achievements
                .show()
                .map_err(|e| anyhow::anyhow!("Failed to send achievement notification: {}", e))?;
            
            println!("Achievement notification: {}", message);
        }
        
        Ok(())
    }

    fn is_daily_reset_time(&self) -> bool {
        let now = Local::now().time();
        let reset_time = match self.config.get_reset_time() {
            Ok(time) => time,
            Err(_) => return false,
        };
        
        // Check if we're within 1 minute of reset time
        let diff = if now >= reset_time {
            now - reset_time
        } else {
            reset_time - now
        };
        
        diff <= Duration::minutes(1)
    }

    fn send_daily_reset_notification(&self) -> Result<()> {
        // Play notification sound
        self.play_notification_sound();

        Notification::new()
            .summary("ThreeADay - New Day!")
            .body("🌅 Ready for a fresh start? Add your first task for today!")
            .urgency(Urgency::Critical)
            .timeout(0) // Persistent notification
            .show()
            .map_err(|e| anyhow::anyhow!("Failed to send daily reset notification: {}", e))?;
        
        println!("Sent persistent daily reset notification");
        Ok(())
    }

    fn play_notification_sound(&self) {
        // Try multiple common notification sounds, starting with system default
        let sound_paths = [
            "/usr/share/sounds/freedesktop/stereo/message-new-instant.oga",
            "/usr/share/sounds/freedesktop/stereo/bell.oga",
            "/usr/share/sounds/alarms/Oxygen-Sys-App-Message.ogg",
            "/usr/share/sounds/ubuntu/stereo/message-new-instant.ogg",
            "/usr/share/sounds/generic/click.wav",
        ];

        for sound_path in &sound_paths {
            if std::path::Path::new(sound_path).exists() {
                // Try paplay first (PulseAudio)
                if let Ok(_) = Command::new("paplay").arg(sound_path).output() {
                    return;
                }
                // Fallback to aplay (ALSA)
                if let Ok(_) = Command::new("aplay").arg(sound_path).output() {
                    return;
                }
            }
        }

        // Final fallback: system bell
        let _ = Command::new("bash").arg("-c").arg(r"echo -e '\a'").output();
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting ThreeADay service...");
    
    let mut state = ServiceState::new()?;
    
    // Check for achievements on startup
    state.check_for_achievements()?;
    
    // Main service loop
    let mut reminder_interval = interval(StdDuration::from_secs(
        state.config.reminder_interval_minutes * 60
    ));
    
    // Check every minute for daily reset
    let mut daily_check_interval = interval(StdDuration::from_secs(60));
    
    loop {
        tokio::select! {
            _ = reminder_interval.tick() => {
                if state.is_new_day() {
                    state.reset_daily_counters();
                }
                
                if state.should_send_reminder() {
                    if let Err(e) = state.send_reminder() {
                        eprintln!("Error sending reminder: {}", e);
                    }
                }
            }
            
            _ = daily_check_interval.tick() => {
                if state.is_new_day() {
                    state.reset_daily_counters();
                }
                
                if state.is_daily_reset_time() {
                    if let Err(e) = state.send_daily_reset_notification() {
                        eprintln!("Error sending daily reset notification: {}", e);
                    }
                }
            }
        }
    }
}