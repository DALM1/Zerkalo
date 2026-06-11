#!/bin/bash

set -e

PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"
BINARY_PATH="$PROJECT_DIR/target/release/zerkalo"
PLIST_DIR="$HOME/Library/LaunchAgents"
PLIST_PATH="$PLIST_DIR/com.zerkalo.daemon.plist"
LOG_DIR="$HOME/Library/Logs/Zerkalo"
STDOUT_LOG="$LOG_DIR/stdout.log"
STDERR_LOG="$LOG_DIR/stderr.log"

if [ ! -f "$BINARY_PATH" ]; then
    echo "⚫️ Error: zerkalo binary not found at $BINARY_PATH"
    exit 1
fi

mkdir -p "$PLIST_DIR" "$LOG_DIR"

cat > "$PLIST_PATH" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.zerkalo.daemon</string>

    <key>ProgramArguments</key>
    <array>
        <string>$BINARY_PATH</string>
    </array>

    <key>RunAtLoad</key>
    <true/>

    <key>KeepAlive</key>
    <true/>

    <key>WorkingDirectory</key>
    <string>$PROJECT_DIR</string>

    <key>StandardOutPath</key>
    <string>$STDOUT_LOG</string>

    <key>StandardErrorPath</key>
    <string>$STDERR_LOG</string>
</dict>
</plist>
EOF

launchctl bootout "gui/$(id -u)/com.zerkalo.daemon" >/dev/null 2>&1 || true
launchctl bootstrap "gui/$(id -u)" "$PLIST_PATH"
launchctl kickstart -k "gui/$(id -u)/com.zerkalo.daemon"

echo "⚪️ LaunchAgent installed at $PLIST_PATH"
echo "⚪️ Zerkalo will now start automatically at login."
echo "⚪️ If permissions are still missing, add the launched binary to Accessibility if needed."
