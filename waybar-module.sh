#!/bin/bash

# ThreeADay waybar module
# Shows task progress and allows clicking to open GUI

get_status() {
    # Use the CLI to get status, parse the output
    local output=$(threeaday status 2>/dev/null)
    
    if [[ $? -ne 0 ]]; then
        echo '{"text": "❌", "tooltip": "ThreeADay: Error getting status", "class": "error"}'
        return
    fi
    
    # Parse "Today's progress: X/Y tasks completed"
    if [[ $output =~ Today\'s\ progress:\ ([0-9]+)/([0-9]+) ]]; then
        local completed=${BASH_REMATCH[1]}
        local total=${BASH_REMATCH[2]}
        
        # Determine display text and class
        local text
        local class="normal"
        local tooltip="ThreeADay: $completed/$total tasks completed"
        
        if [[ $completed -ge 3 ]]; then
            text="🎯 $completed"
            class="achieved"
            tooltip="ThreeADay: Goal achieved! $completed tasks completed"
        elif [[ $total -eq 0 ]]; then
            text="📝 0"
            class="empty"
            tooltip="ThreeADay: No tasks yet. Click to add some!"
        else
            text="📋 $completed/$total"
            if [[ $completed -eq 0 ]]; then
                class="none"
            elif [[ $completed -eq 1 ]]; then
                class="started"
            else
                class="progress"
            fi
            tooltip="ThreeADay: $completed/$total tasks completed. $(( 3 - completed )) more to go!"
        fi
        
        echo "{\"text\": \"$text\", \"tooltip\": \"$tooltip\", \"class\": \"$class\"}"
    else
        # Fallback if parsing fails
        echo '{"text": "📋 ?", "tooltip": "ThreeADay: Status unknown", "class": "unknown"}'
    fi
}

handle_click() {
    case "$1" in
        "left")
            # Left click: open GUI
            threeaday gui >/dev/null 2>&1 &
            ;;
        "right")
            # Right click: show today's tasks in notification
            local tasks=$(threeaday list 2>/dev/null)
            if [[ $? -eq 0 ]]; then
                notify-send "ThreeADay Tasks" "$tasks" -t 5000
            else
                notify-send "ThreeADay" "Error getting task list" -t 3000
            fi
            ;;
        "middle")
            # Middle click: quick add via zenity/rofi if available
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
                notify-send "ThreeADay" "Middle click: Install rofi or zenity for quick add" -t 3000
            fi
            ;;
    esac
}

# Handle arguments
case "${1:-status}" in
    "status")
        get_status
        ;;
    "click")
        handle_click "$2"
        ;;
    *)
        get_status
        ;;
esac