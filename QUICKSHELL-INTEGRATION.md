# QuickShell Integration for ThreeADay

This guide shows how to integrate ThreeADay with quickshell (or other similar shell-based status bars).

## Quick Start

The `quickshell-widget-example.sh` script provides a simple interface to display ThreeADay progress in your status bar.

### Installation

```bash
# Make the script executable
chmod +x quickshell-widget-example.sh

# Copy it to your config directory (or anywhere in your PATH)
cp quickshell-widget-example.sh ~/.config/threeaday/widget.sh
```

### Basic Usage

The script supports several commands:

```bash
# Get display text (shows emoji + progress)
./quickshell-widget-example.sh text
# Output: "📋 1/2" or "🎯 3" or "📝 0"

# Get tooltip (detailed status)
./quickshell-widget-example.sh tooltip
# Output: "ThreeADay: 1/2 tasks completed. 2 more to go!"

# Get color (for styling based on progress)
./quickshell-widget-example.sh color
# Output: "#ff6b6b" (red), "#ffa500" (orange), "#ffd700" (yellow), or "#51cf66" (green)

# Handle clicks
./quickshell-widget-example.sh click left    # Opens GUI
./quickshell-widget-example.sh click right   # Shows task list in notification
./quickshell-widget-example.sh click middle  # Quick add (rofi/zenity)
```

## QuickShell Integration Example

Here's a basic example of how you might integrate this into quickshell.
The exact syntax will depend on your quickshell configuration style:

```qml
// Example quickshell widget (adapt to your config format)
ShellCommand {
    command: "~/.config/threeaday/widget.sh text"
    interval: 30000  // Update every 30 seconds

    onClicked: function(button) {
        if (button === Qt.LeftButton) {
            ShellCommand.run("~/.config/threeaday/widget.sh click left");
        } else if (button === Qt.RightButton) {
            ShellCommand.run("~/.config/threeaday/widget.sh click right");
        } else if (button === Qt.MiddleButton) {
            ShellCommand.run("~/.config/threeaday/widget.sh click middle");
        }
    }
}
```

## Color Coding

The widget uses colors to indicate progress:

- 🔴 **Red** (#ff6b6b) - 0 tasks completed - Need to start!
- 🟠 **Orange** (#ffa500) - 1 task completed - Making progress
- 🟡 **Yellow** (#ffd700) - 2 tasks completed - Almost there!
- 🟢 **Green** (#51cf66) - 3+ tasks completed - Goal achieved!

## Customization

You can customize the script by editing `quickshell-widget-example.sh`:

- **Change emoji icons**: Edit the `get_status()` function
- **Modify colors**: Edit the `get_color()` function
- **Add new click actions**: Edit the `handle_click()` function
- **Change update interval**: Adjust in your quickshell config

## Alternative: Simple Polling Widget

If your status bar doesn't support shell scripts, you can create a simple polling widget:

```bash
#!/bin/bash
# Simple polling version
while true; do
    threeaday status | grep "Today's progress" | sed 's/Today.s progress: /📋 /'
    sleep 30
done
```

## Troubleshooting

**Widget shows "📋 ?":**
- Check that `threeaday` is in your PATH
- Verify the service is running: `systemctl --user status threeaday`

**Click actions don't work:**
- Ensure the script has execute permissions: `chmod +x widget.sh`
- For quick-add, install either `rofi` or `zenity`

**Widget doesn't update:**
- Check your quickshell update interval
- Try manually running: `~/.config/threeaday/widget.sh text`

## See Also

- `waybar-module.sh` - Similar integration for waybar
- `threeaday status` - CLI command for getting current progress
- `threeaday --help` - Full CLI documentation
