import QtQuick
import Quickshell.Io
import qs.Common
import qs.Widgets
import qs.Modules.Plugins

PluginComponent {
    id: root

    // State
    property int completed: 0
    property int total: 0
    property bool goalAchieved: completed >= 3
    property string currentOutput: "..."
    property bool statusParsed: false

    // Paths
    readonly property string threeadayBin: "/home/carlose/.cargo/bin/threeaday"
    readonly property string threeadayGui: "/home/carlose/.cargo/bin/threeaday-gui"

    // Computed display text
    readonly property string displayText: {
        if (goalAchieved) {
            return completed.toString()
        } else if (total === 0) {
            return "0"
        } else {
            return completed + "/" + total
        }
    }

    // Computed icon based on progress
    readonly property string displayIcon: {
        if (goalAchieved) {
            return "emoji_events"
        } else if (total === 0) {
            return "edit_note"
        } else {
            return "checklist"
        }
    }

    // Computed color based on progress
    readonly property color progressColor: {
        if (goalAchieved) {
            return Theme.success || "#51cf66"
        } else if (completed === 2) {
            return Theme.warning || "#ffd700"
        } else if (completed === 1) {
            return Theme.caution || "#ffa500"
        } else {
            return Theme.error || "#ff6b6b"
        }
    }

    Component.onCompleted: {
        refreshStatus()
    }

    Timer {
        id: pollTimer
        interval: 30000
        repeat: true
        running: true
        onTriggered: root.refreshStatus()
    }

    function refreshStatus() {
        root.statusParsed = false
        statusProcess.running = true
    }

    function parseStatusLine(line) {
        // Only parse once per refresh cycle
        if (root.statusParsed) return

        var match = line.match(/Today's progress: (\d+)\/(\d+)/)
        if (match) {
            root.completed = parseInt(match[1])
            root.total = parseInt(match[2])
            root.currentOutput = root.displayText
            root.statusParsed = true
        }
    }

    Process {
        id: statusProcess
        command: ["sh", "-c", root.threeadayBin + " status 2>/dev/null"]
        running: false

        stdout: SplitParser {
            onRead: data => {
                root.parseStatusLine(data)
            }
        }

        onExited: (exitCode, exitStatus) => {
            if (exitCode !== 0 && !root.statusParsed) {
                root.currentOutput = "?"
            }
        }
    }

    Process {
        id: actionProcess
        command: ["sh", "-c", ""]
        running: false

        onExited: (exitCode, exitStatus) => {
            // Refresh status after any action
            Qt.callLater(root.refreshStatus)
        }
    }

    // Left click opens GUI
    pillClickAction: () => {
        actionProcess.command = ["setsid", root.threeadayGui]
        actionProcess.running = true
    }

    horizontalBarPill: Component {
        MouseArea {
            implicitWidth: contentRow.implicitWidth
            implicitHeight: contentRow.implicitHeight
            acceptedButtons: Qt.MiddleButton | Qt.RightButton
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor

            onClicked: mouse => {
                if (mouse.button === Qt.RightButton) {
                    // Show tasks via notification - proper escaping
                    actionProcess.command = ["sh", "-c",
                        "tasks=$(" + root.threeadayBin + " list 2>/dev/null) && notify-send -u normal -t 8000 'ThreeADay Tasks' \"$tasks\""]
                    actionProcess.running = true
                } else if (mouse.button === Qt.MiddleButton) {
                    // Middle click: quick add via wofi
                    actionProcess.command = ["sh", "-c",
                        "task=$(echo '' | wofi -d -p 'Add task:' 2>/dev/null) && " +
                        "[ -n \"$task\" ] && " + root.threeadayBin + " add \"$task\" && " +
                        "notify-send -u normal -t 2000 'ThreeADay' \"Added: $task\""]
                    actionProcess.running = true
                }
            }

            Row {
                id: contentRow
                spacing: Theme.spacingXS

                DankIcon {
                    name: root.displayIcon
                    size: Theme.iconSize - 6
                    color: root.progressColor
                    anchors.verticalCenter: parent.verticalCenter
                }

                StyledText {
                    text: root.currentOutput
                    font.pixelSize: Theme.fontSizeSmall
                    font.weight: Font.Medium
                    color: root.progressColor
                    anchors.verticalCenter: parent.verticalCenter
                }
            }
        }
    }

    verticalBarPill: Component {
        MouseArea {
            implicitWidth: contentColumn.implicitWidth
            implicitHeight: contentColumn.implicitHeight
            acceptedButtons: Qt.MiddleButton | Qt.RightButton
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor

            onClicked: mouse => {
                if (mouse.button === Qt.RightButton) {
                    actionProcess.command = ["sh", "-c",
                        "tasks=$(" + root.threeadayBin + " list 2>/dev/null) && notify-send -u normal -t 8000 'ThreeADay Tasks' \"$tasks\""]
                    actionProcess.running = true
                } else if (mouse.button === Qt.MiddleButton) {
                    actionProcess.command = ["sh", "-c",
                        "if command -v fuzzel >/dev/null 2>&1; then " +
                        "task=$(echo '' | fuzzel -d -p 'Add task: ' 2>/dev/null) && " +
                        "[ -n \"$task\" ] && " + root.threeadayBin + " add \"$task\" && " +
                        "notify-send -u normal -t 2000 'ThreeADay' \"Added: $task\"; " +
                        "else " + root.threeadayGui + "; fi"]
                    actionProcess.running = true
                }
            }

            Column {
                id: contentColumn
                spacing: Theme.spacingXS

                DankIcon {
                    name: root.displayIcon
                    size: Theme.iconSize - 6
                    color: root.progressColor
                    anchors.horizontalCenter: parent.horizontalCenter
                }

                StyledText {
                    text: root.currentOutput
                    font.pixelSize: Theme.fontSizeSmall
                    font.weight: Font.Medium
                    color: root.progressColor
                    anchors.horizontalCenter: parent.horizontalCenter
                }
            }
        }
    }
}
