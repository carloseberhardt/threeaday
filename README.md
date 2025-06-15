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
```

### Service Management
```bash
threeaday start-service                 # Start background service
threeaday stop-service                  # Stop background service
threeaday service-status                # Check service status
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

## Architecture

- **CLI** (`threeaday`): Task management interface
- **Service** (`threeaday-service`): Background notifications via mako
- **GUI** (`threeaday-gui`): GTK4 interface  
- **Database**: SQLite storage in `~/.local/share/threeaday/`
- **Waybar Module**: Shell script for status display

## Design Philosophy

- **3 tasks maximum** - prevents overwhelm
- **Daily reset at 6 AM** - fresh start each day
- **Encouraging feedback** - celebrates progress
- **Minimal friction** - quick to add/complete tasks
- **Always visible** - waybar integration keeps it present

Built with Rust for reliability and performance.