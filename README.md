# ThreeADay

A momentum-building app to break out of depression and stagnation by focusing on completing just 3 small tasks each day.

## Features

- **CLI**: Quick task management from the terminal
- **GUI**: Clean, simple interface with progress tracking  
- **Service**: Background notifications and daily resets
- **Waybar**: Always-visible progress indicator with click actions

## Installation

```bash
# Clone and install
git clone <repo-url>
cd threeaday
./install.sh
```

## Usage

### CLI Commands
```bash
threeaday add "take a 5 minute walk"    # Add a task
threeaday list                          # List today's tasks  
threeaday done 1                        # Complete task ID 1
threeaday status                        # Check progress
threeaday gui                           # Launch GUI
threeaday config                        # Show config file location
```

### Service Management
```bash
threeaday start-service                 # Start background service
threeaday stop-service                  # Stop background service
threeaday service-status                # Check service status
```

### Configuration

Service behavior can be customized via `~/.config/threeaday/config.toml`:

```toml
# How often to send reminder notifications (in minutes)
reminder_interval_minutes = 45

# Daily reset time in 24-hour format (HH:MM) 
daily_reset_time = "06:00"

# Maximum number of reminders per day
max_reminders_per_day = 8
```

After editing config, restart the service:
```bash
threeaday stop-service && threeaday start-service
```

### Waybar Integration

Add to your waybar config:
```json
"threeaday": {
    "format": "{}",
    "return-type": "json", 
    "exec": "~/.config/threeaday/waybar-module.sh status",
    "on-click": "~/.config/threeaday/waybar-module.sh click left",
    "on-click-right": "~/.config/threeaday/waybar-module.sh click right", 
    "on-click-middle": "~/.config/threeaday/waybar-module.sh click middle",
    "interval": 30,
    "tooltip": true
}
```

**Click Actions:**
- **Left click**: Open GUI
- **Right click**: Show task list notification
- **Middle click**: Quick add task (requires rofi or zenity)

### GUI Features

The GTK4 interface provides:
- **Visual task management** with checkboxes and progress tracking
- **3-task focus** - GUI shows max 3 tasks and prevents adding more
- **Achievement celebrations** with animated notifications when goal reached
- **Keyboard shortcuts**: Enter to add tasks, Escape to close window
- **Auto-refresh** when tasks are completed

## Architecture

- **CLI** (`threeaday`): Task management interface
- **Service** (`threeaday-service`): Background notifications via mako
- **GUI** (`threeaday-gui`): GTK4 interface  
- **Database**: SQLite storage in `~/.local/share/threeaday/`
- **Config**: TOML file in `~/.config/threeaday/config.toml`
- **Waybar Module**: Shell script for status display

## Design Philosophy

- **3 tasks maximum** - prevents overwhelm
- **Daily fresh start** - tasks don't carry over, configurable reset time (default 6 AM)
- **Encouraging feedback** - celebrates progress with notifications and animations
- **Minimal friction** - quick to add/complete tasks from CLI or GUI
- **Always visible** - waybar integration keeps progress present
- **Persistent reminders** - configurable notifications until goal achieved
- **Focus over features** - simple, effective momentum building

Built with Rust for reliability and performance.