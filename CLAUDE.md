# ThreeADay - Design Decisions & Development Notes

This file captures key design decisions, philosophy, and development context for ThreeADay.

## Core Philosophy

**Problem**: Depression and stagnation often come from overwhelming todo lists and lack of momentum.

**Solution**: Focus on completing just 3 small tasks daily to build sustainable momentum without overwhelm.

### Key Principles
1. **Small scope = achievable goals** - Never more than 3 daily tasks in focus
2. **Fresh start daily** - No guilt from yesterday's failures, tasks don't carry over
3. **Persistent but gentle encouragement** - Configurable reminders until goal achieved
4. **Always present** - Waybar integration keeps progress visible
5. **Multiple interfaces** - CLI for power users, GUI for visual appeal, service for automation

## Architecture Decisions

### Multi-Binary Approach
- **`threeaday`** (CLI) - Primary interface, fast task management
- **`threeaday-service`** - Background daemon for notifications/resets
- **`threeaday-gui`** - GTK4 visual interface
- **Rationale**: Different use cases require different tools, but shared database

### Database: SQLite
- **Choice**: SQLite with date-based task storage
- **Why**: Simple, reliable, no server needed, handles concurrent access
- **Schema**: Tasks tied to creation date, no carryover between days
- **Location**: `~/.local/share/threeaday/tasks.db`

### Task Lifecycle
- **Daily reset**: Configurable time (default 6 AM) via `daily_reset_time`
- **No carryover**: Incomplete tasks disappear at reset (intentional for mental health)
- **ID assignment**: Sequential within each day

### Configuration
- **File**: `~/.config/threeaday/config.toml`
- **Auto-creation**: Service creates default config on first run
- **Key settings**:
  - `reminder_interval_minutes` - Notification frequency (default: 45)
  - `daily_reset_time` - When new day starts (default: "06:00")
  - `max_reminders_per_day` - Spam prevention (default: 8)

## Interface Design Decisions

### CLI
- **Power user focused** - No limits, can add >3 tasks if needed
- **Quick actions** - `add`, `list`, `done`, `status` commands
- **Service management** - Control background daemon
- **Config access** - `config` command shows file location

### GUI (GTK4)
- **3-task maximum** - Enforces core philosophy visually
- **Gentle limits** - Shows "Focus on your 3 tasks first!" if trying to add 4th
- **Overflow indicator** - "... and X more tasks (use CLI to see all)"
- **Immediate feedback** - Achievement celebrations, progress tracking
- **Keyboard shortcuts**: 
  - Enter: Add task
  - Escape: Close window
- **Auto-focus** - Window gets focus for immediate keyboard use

### Service (Background Daemon)
- **Smart reminders** - Only when needed, stops at goal achievement
- **Persistent encouragement** - Will remind even with 0 tasks (changed from original)
- **Achievement celebrations** - Special notifications for 3+ completions
- **Daily reset notifications** - 6 AM "fresh start" encouragement
- **mako integration** - Uses desktop notification system

### Waybar Integration
- **Always visible** - Progress always present in status bar
- **Color coding**:
  - Red (none/0): Need to start
  - Orange (started/1): Making progress  
  - Yellow (progress/2): Almost there
  - Green (achieved/3+): Goal reached with glow animation
- **Click actions**:
  - Left: Open GUI
  - Right: Show task list notification
  - Middle: Quick add (rofi/zenity)

## Technical Decisions

### Language: Rust
- **Why**: Reliability, performance, single binary deployment
- **GTK4-rs**: Mature bindings for native GUI
- **tokio**: Async runtime for service
- **systemd**: User service for auto-start

### Testing Strategy
- **Unit tests**: Database operations (5 tests)
- **Integration tests**: CLI commands end-to-end (7 tests)
- **No GUI tests**: Too complex for personal project, manual testing sufficient
- **Test isolation**: Each test uses separate database via env vars

### Error Handling
- **anyhow**: Consistent error handling across all binaries
- **Graceful degradation**: Service continues on notification failures
- **User-friendly messages**: CLI shows helpful error messages

## User Experience Decisions

### Momentum Building Focus
- **No overwhelming features** - Deliberately simple
- **Positive reinforcement** - Celebrations, encouraging messages
- **Remove friction** - Quick CLI commands, GUI shortcuts
- **Prevent guilt spirals** - Daily reset, no carryover

### Flexibility vs. Constraints
- **GUI enforces 3-task limit** - Visual constraint for focus
- **CLI allows more** - Power users can bypass if needed
- **Configurable timing** - Personal schedules vary
- **Escape hatches** - Always a way to work around limits

## Installation & Distribution
- **install.sh**: Automated setup for EndeavourOS/systemd
- **systemd user service**: Auto-start with desktop session
- **waybar integration**: Separate module script
- **MIT license**: Maximum sharing freedom

## Future Considerations

### Potential Enhancements
- Streak tracking (but avoid guilt on breaks)
- Time estimates for tasks (but keep simple)
- Categories/tags (but avoid over-organization)
- Weekly/monthly views (but maintain daily focus)

### Things to Avoid
- Complex project management features
- Due dates or scheduling (creates pressure)
- Detailed analytics (can become obsessive)
- Syncing/cloud features (adds complexity)

### Principles to Maintain
- Keep it simple and focused
- Always prioritize momentum over features
- Preserve the daily reset philosophy
- Don't let it become another overwhelming todo app

## Development Environment
- **Target platform**: EndeavourOS with Hyprland + Waybar
- **Dependencies**: GTK4, mako notifications, systemd
- **Build**: Standard Cargo workflow
- **Testing**: `cargo test -- --test-threads=1` (database isolation)

---

*This philosophy emerged from real experience with depression and todo list paralysis. The 3-task limit isn't arbitrary - it's the sweet spot between progress and overwhelm.*