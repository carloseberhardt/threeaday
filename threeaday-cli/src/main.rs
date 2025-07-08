use clap::{Parser, Subcommand};
use std::process;
use threeaday_core::{Database, Config, Result, utils::*};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new task
    Add { text: String },
    /// List today's tasks
    List,
    /// Mark a task as completed
    Done { id: i64 },
    /// Show today's progress
    Status,
    /// Launch the GUI
    Gui,
    /// Show config file location
    Config,
    /// Start the background service
    StartService,
    /// Stop the background service
    StopService,
    /// Check service status
    ServiceStatus,
}

fn main() {
    let cli = Cli::parse();
    
    if let Err(e) = run(cli) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Add { text } => {
            let mut db = Database::new()?;
            let task_id = db.add_task(&text)?;
            println!("Added task {} with ID {}", text, task_id);
        }
        Commands::List => {
            let db = Database::new()?;
            let tasks = db.get_today_tasks()?;
            
            if tasks.is_empty() {
                println!("No tasks for today yet. Add some with 'threeaday add \"task text\"'");
            } else {
                println!("Today's tasks:");
                for task in tasks {
                    let status = format_task_status(task.completed);
                    println!("  [{}] {}: {}", status, task.id, task.text);
                }
            }
        }
        Commands::Done { id } => {
            let mut db = Database::new()?;
            if db.complete_task(id)? {
                println!("Task {} completed! 🎉", id);
                
                // Check if goal is achieved
                let (completed, _total) = db.get_today_summary()?;
                if is_daily_goal_achieved(completed) {
                    println!("🎯 Daily goal achieved! You completed {} tasks today!", completed);
                }
            } else {
                println!("Task {} not found or already completed", id);
            }
        }
        Commands::Status => {
            let db = Database::new()?;
            let (completed, total) = db.get_today_summary()?;
            println!("Today's progress: {}/{} tasks completed", completed, total);
            
            if is_daily_goal_achieved(completed) {
                println!("🎯 Daily goal achieved! Great job!");
            } else {
                let remaining = DAILY_GOAL_COMPLETION_COUNT.saturating_sub(completed);
                println!("You need {} more task(s) to reach your daily goal of {}", remaining, DAILY_GOAL_COMPLETION_COUNT);
            }
        }
        Commands::Gui => {
            // Launch GUI in background
            let output = std::process::Command::new("threeaday-gui")
                .spawn()?;
            println!("GUI launched (PID: {})", output.id());
        }
        Commands::Config => {
            let config_path = Config::get_config_path()?;
            println!("Config file location: {}", config_path.display());
            
            if config_path.exists() {
                println!("Config file exists and can be edited.");
            } else {
                println!("Config file does not exist yet - it will be created when the service starts.");
            }
        }
        Commands::StartService => {
            let output = std::process::Command::new("systemctl")
                .args(&["--user", "start", "threeaday"])
                .output()?;
            
            if output.status.success() {
                println!("Service started successfully");
            } else {
                eprintln!("Failed to start service: {}", String::from_utf8_lossy(&output.stderr));
            }
        }
        Commands::StopService => {
            let output = std::process::Command::new("systemctl")
                .args(&["--user", "stop", "threeaday"])
                .output()?;
            
            if output.status.success() {
                println!("Service stopped successfully");
            } else {
                eprintln!("Failed to stop service: {}", String::from_utf8_lossy(&output.stderr));
            }
        }
        Commands::ServiceStatus => {
            let output = std::process::Command::new("systemctl")
                .args(&["--user", "status", "threeaday"])
                .output()?;
            
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }
    }
    
    Ok(())
}

