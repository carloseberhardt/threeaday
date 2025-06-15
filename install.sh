#!/bin/bash
set -e

echo "Installing ThreeADay..."

# Build the project
cargo build --release

# Install binaries to ~/.cargo/bin (or create symlinks)
mkdir -p ~/.cargo/bin
cp target/release/threeaday ~/.cargo/bin/
cp target/release/threeaday-service ~/.cargo/bin/
cp target/release/threeaday-gui ~/.cargo/bin/

# Install waybar module
mkdir -p ~/.config/threeaday
cp waybar-module.sh ~/.config/threeaday/
chmod +x ~/.config/threeaday/waybar-module.sh

# Install systemd user service
mkdir -p ~/.config/systemd/user
cp threeaday.service ~/.config/systemd/user/

# Reload systemd and enable the service
systemctl --user daemon-reload
systemctl --user enable threeaday.service

echo "✅ Installation complete!"
echo ""
echo "Usage:"
echo "  threeaday add \"your task\"     # Add a task"
echo "  threeaday list                 # List today's tasks"
echo "  threeaday done <id>            # Complete a task"
echo "  threeaday status               # Check progress"
echo "  threeaday gui                  # Launch GUI"
echo ""
echo "Service management:"
echo "  systemctl --user start threeaday    # Start service"
echo "  systemctl --user stop threeaday     # Stop service" 
echo "  systemctl --user status threeaday   # Check service status"
echo ""
echo "Waybar integration:"
echo "  Module script installed to: ~/.config/threeaday/waybar-module.sh"
echo "  Add this to your waybar config:"
echo "    \"threeaday\": {"
echo "      \"format\": \"{}\","
echo "      \"return-type\": \"json\","
echo "      \"exec\": \"~/.config/threeaday/waybar-module.sh status\","
echo "      \"on-click\": \"~/.config/threeaday/waybar-module.sh click left\","
echo "      \"on-click-right\": \"~/.config/threeaday/waybar-module.sh click right\","
echo "      \"on-click-middle\": \"~/.config/threeaday/waybar-module.sh click middle\","
echo "      \"interval\": 30,"
echo "      \"tooltip\": true"
echo "    }"
echo ""
echo "The service will automatically start reminders and handle daily resets."