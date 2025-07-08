use threeaday_core::{Database, Config, Result, utils::*};
use notify_rust::Notification;
use std::time::Duration;
use tokio::time::{sleep, Instant};
use chrono::Timelike;

struct ServiceState {
    db: Database,
    config: Config,
    last_reminder_time: Option<Instant>,
    last_reset_time: Option<Instant>,
    reminders_sent_today: u32,
}

impl ServiceState {
    fn new() -> Result<Self> {
        let db = Database::new()?;
        let config = Config::load()?;
        
        Ok(ServiceState {
            db,
            config,
            last_reminder_time: None,
            last_reset_time: None,
            reminders_sent_today: 0,
        })
    }

    fn should_send_reminder(&self) -> bool {
        if self.reminders_sent_today >= self.config.max_reminders_per_day {
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

        self.last_reminder_time = Some(Instant::now());
        self.reminders_sent_today += 1;

        Ok(())
    }

    fn should_send_daily_reset(&self) -> bool {
        // Check if it's time for daily reset (6 AM by default)
        let now = chrono::Utc::now().time();
        let reset_time = chrono::NaiveTime::parse_from_str(&self.config.daily_reset_time, "%H:%M")
            .unwrap_or_else(|_| chrono::NaiveTime::from_hms_opt(6, 0, 0).unwrap());
        
        // Simple check: if it's past reset time and we haven't sent today's reset
        if now.hour() >= reset_time.hour() && now.minute() >= reset_time.minute() {
            if let Some(last_reset) = self.last_reset_time {
                // Don't send another reset if we sent one in the last hour
                return last_reset.elapsed().as_secs() > 3600;
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

        self.last_reset_time = Some(Instant::now());
        self.reminders_sent_today = 0; // Reset reminder counter

        Ok(())
    }

    fn send_achievement_notification(&self) -> Result<()> {
        let (completed, _total) = self.db.get_today_summary()?;
        
        if is_daily_goal_achieved(completed) {
            let message = format!("🎉 Congratulations! You've completed {} tasks today. Goal achieved! 🎯", completed);
            
            Notification::new()
                .summary("ThreeADay - Goal Achieved!")
                .body(&message)
                .timeout(8000)
                .show()?;
        }
        
        Ok(())
    }

    async fn run_main_loop(&mut self) -> Result<()> {
        let mut last_task_check = Instant::now();
        let mut last_achievement_check: Option<Instant> = None;
        
        loop {
            let now = Instant::now();
            
            // Check for daily reset
            if let Err(e) = self.send_daily_reset_notification() {
                eprintln!("Error sending daily reset notification: {}", e);
            }
            
            // Check for reminders (every configured interval)
            if let Some(last_reminder) = self.last_reminder_time {
                if last_reminder.elapsed().as_secs() >= self.config.reminder_interval_minutes * 60 {
                    if let Err(e) = self.send_reminder() {
                        eprintln!("Error sending reminder: {}", e);
                    }
                }
            } else {
                // Send first reminder
                if let Err(e) = self.send_reminder() {
                    eprintln!("Error sending reminder: {}", e);
                }
            }
            
            // Check for achievement notification (when goal is reached)
            if last_task_check.elapsed().as_secs() >= 30 {
                match self.db.get_today_summary() {
                    Ok((completed, _total)) => {
                        if is_daily_goal_achieved(completed) {
                            // Only send achievement notification once per day
                            if last_achievement_check.is_none() 
                                || last_achievement_check.unwrap().elapsed().as_secs() > 86400 {
                                if let Err(e) = self.send_achievement_notification() {
                                    eprintln!("Error sending achievement notification: {}", e);
                                }
                                last_achievement_check = Some(now);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error checking task summary: {}", e);
                    }
                }
                last_task_check = now;
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