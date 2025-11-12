#!/bin/bash

# ThreeADay quickshell widget example
# This script can be called from your quickshell config to display task progress
#
# Usage in quickshell (adapt to your shell's specific syntax):
#   - Text: Call this script with no arguments to get display text
#   - Tooltip: Call with "tooltip" to get detailed info
#   - Click handlers: Call with "click" to open GUI

get_status() {
    # Get status from CLI, parse the output
    local output
    output=$(threeaday status 2>/dev/null)

    if [[ $? -ne 0 ]]; then
        echo "📋 ?"
        return
    fi

    # Parse "Today's progress: X/Y tasks completed"
    if [[ $output =~ Today\'s\ progress:\ ([0-9]+)/([0-9]+) ]]; then
        local completed=${BASH_REMATCH[1]}
        local total=${BASH_REMATCH[2]}

        # Return text based on progress
        if [[ $completed -ge 3 ]]; then
            echo "🎯 $completed"
        elif [[ $total -eq 0 ]]; then
            echo "📝 0"
        else
            echo "📋 $completed/$total"
        fi
    else
        echo "📋 ?"
    fi
}

get_tooltip() {
    local output
    output=$(threeaday status 2>/dev/null)

    if [[ $? -ne 0 ]]; then
        echo "ThreeADay: Status unknown"
        return
    fi

    if [[ $output =~ Today\'s\ progress:\ ([0-9]+)/([0-9]+) ]]; then
        local completed=${BASH_REMATCH[1]}
        local total=${BASH_REMATCH[2]}

        if [[ $completed -ge 3 ]]; then
            echo "ThreeADay: Goal achieved! $completed tasks completed"
        elif [[ $total -eq 0 ]]; then
            echo "ThreeADay: No tasks yet. Click to add some!"
        else
            local remaining=$((3 - completed))
            echo "ThreeADay: $completed/$total tasks completed. $remaining more to go!"
        fi
    else
        echo "ThreeADay: Status unknown"
    fi
}

get_color() {
    local output
    output=$(threeaday status 2>/dev/null)

    if [[ $? -ne 0 ]]; then
        echo "#888888"  # gray
        return
    fi

    if [[ $output =~ Today\'s\ progress:\ ([0-9]+)/([0-9]+) ]]; then
        local completed=${BASH_REMATCH[1]}

        case $completed in
            0)
                echo "#ff6b6b"  # red - nothing done
                ;;
            1)
                echo "#ffa500"  # orange - started
                ;;
            2)
                echo "#ffd700"  # yellow - almost there
                ;;
            *)
                echo "#51cf66"  # green - goal achieved!
                ;;
        esac
    else
        echo "#888888"  # gray
    fi
}

handle_click() {
    case "$1" in
        "left")
            # Left click: open GUI
            threeaday gui >/dev/null 2>&1 &
            ;;
        "right")
            # Right click: show task list in notification
            local tasks=$(threeaday list 2>/dev/null)
            if [[ $? -eq 0 ]]; then
                notify-send "ThreeADay Tasks" "$tasks" -t 5000
            fi
            ;;
        "middle")
            # Middle click: quick add (you can customize this)
            if command -v rofi >/dev/null; then
                local task=$(echo "" | rofi -dmenu -p "Add task:" -lines 0)
                if [[ -n "$task" ]]; then
                    threeaday add "$task"
                    notify-send "ThreeADay" "Added: $task" -t 2000
                fi
            elif command -v zenity >/dev/null; then
                local task=$(zenity --entry --title="ThreeADay" --text="Add a task:")
                if [[ -n "$task" ]]; then
                    threeaday add "$task"
                    notify-send "ThreeADay" "Added: $task" -t 2000
                fi
            else
                # Fallback: just open the GUI
                threeaday gui >/dev/null 2>&1 &
            fi
            ;;
    esac
}

# Handle command
case "${1:-text}" in
    "text"|"status")
        get_status
        ;;
    "tooltip")
        get_tooltip
        ;;
    "color")
        get_color
        ;;
    "click")
        handle_click "$2"
        ;;
    *)
        get_status
        ;;
esac
