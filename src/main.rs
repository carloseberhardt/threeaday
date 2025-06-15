mod db;

use anyhow::Result;
use clap::{Parser, Subcommand};
use db::Database;
use directories::ProjectDirs;

#[derive(Parser)]
#[command(name = "threeaday")]
#[command(about = "A simple momentum-building app for daily progress")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Add a new task")]
    Add {
        #[arg(help = "Task description")]
        text: String,
    },
    #[command(about = "List today's tasks")]
    List,
    #[command(about = "Complete a task by ID")]
    Done {
        #[arg(help = "Task ID to complete")]
        id: i64,
    },
    #[command(about = "Show today's progress summary")]
    Status,
    #[command(about = "Start the background service")]
    StartService,
    #[command(about = "Stop the background service")]
    StopService,
    #[command(about = "Check service status")]
    ServiceStatus,
    #[command(about = "Launch the GUI")]
    Gui,
    #[command(about = "Show service config file location")]
    Config,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut db = Database::new()?;

    match &cli.command {
        Commands::Add { text } => {
            let id = db.add_task(text)?;
            println!("Added task: {} (id: {})", text, id);
        }
        Commands::List => {
            let tasks = db.get_today_tasks()?;
            if tasks.is_empty() {
                println!("No tasks for today. Add some with 'threeaday add \"task description\"'");
            } else {
                for task in tasks {
                    let status = if task.completed { "✓" } else { " " };
                    println!("[{}] {} - {}", status, task.id, task.text);
                }
            }
        }
        Commands::Done { id } => {
            if db.complete_task(*id)? {
                println!("Task {} completed! 🎉", id);
                let (completed, _total) = db.get_today_summary()?;
                if completed >= 3 {
                    println!("Amazing! You've completed {} tasks today!", completed);
                } else {
                    println!("Progress: {}/3 tasks completed", completed);
                }
            } else {
                println!("Task {} not found or already completed", id);
            }
        }
        Commands::Status => {
            let (completed, total) = db.get_today_summary()?;
            println!("Today's progress: {}/{} tasks completed", completed, total);
            if completed >= 3 {
                println!("🎯 Goal achieved! Great momentum!");
            } else if total == 0 {
                println!("💡 Start by adding a task: threeaday add \"your task\"");
            } else {
                println!("🏃 Keep going! {} more to reach your daily goal", 3 - completed);
            }
        }
        Commands::StartService => {
            let output = std::process::Command::new("systemctl")
                .args(["--user", "start", "threeaday"])
                .output()?;
            
            if output.status.success() {
                println!("✅ Service started successfully");
            } else {
                println!("❌ Failed to start service: {}", String::from_utf8_lossy(&output.stderr));
            }
        }
        Commands::StopService => {
            let output = std::process::Command::new("systemctl")
                .args(["--user", "stop", "threeaday"])
                .output()?;
            
            if output.status.success() {
                println!("⏹️ Service stopped successfully");
            } else {
                println!("❌ Failed to stop service: {}", String::from_utf8_lossy(&output.stderr));
            }
        }
        Commands::ServiceStatus => {
            let output = std::process::Command::new("systemctl")
                .args(["--user", "status", "threeaday"])
                .output()?;
            
            println!("{}", String::from_utf8_lossy(&output.stdout));
            if !output.stderr.is_empty() {
                println!("{}", String::from_utf8_lossy(&output.stderr));
            }
        }
        Commands::Gui => {
            let output = std::process::Command::new("threeaday-gui")
                .spawn();
            
            match output {
                Ok(_) => println!("✅ GUI launched"),
                Err(e) => println!("❌ Failed to launch GUI: {}", e),
            }
        }
        Commands::Config => {
            let proj_dirs = ProjectDirs::from("", "", "threeaday")
                .ok_or_else(|| anyhow::anyhow!("Failed to get project directories"))?;
            let config_path = proj_dirs.config_dir().join("config.toml");
            
            println!("Service config file: {}", config_path.display());
            
            if config_path.exists() {
                println!("✅ Config file exists");
                println!("Edit with: $EDITOR '{}'", config_path.display());
            } else {
                println!("ℹ️  Config file will be created automatically when service starts");
                println!("You can also create it now with:");
                println!("mkdir -p {} && cp config-example.toml '{}'", 
                         config_path.parent().unwrap().display(),
                         config_path.display());
            }
            
            println!("\nAfter editing config, restart the service:");
            println!("threeaday stop-service && threeaday start-service");
        }
    }

    Ok(())
}
