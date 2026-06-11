#!/bin/bash

set -e

PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"
APP_DIR="$HOME/Applications/Zerkalo.app"
BINARY_PATH="$APP_DIR/Contents/MacOS/zerkalo"
OPEN_PATH="/usr/bin/open"
PLIST_DIR="$HOME/Library/LaunchAgents"
PLIST_PATH="$PLIST_DIR/com.zerkalo.daemon.plist"
LABEL="com.zerkalo.daemon"
UID_VALUE="$(id -u)"
DOMAIN_TARGET="gui/$UID_VALUE"
LOG_DIR="$HOME/Library/Logs/Zerkalo"
STDOUT_LOG="$LOG_DIR/stdout.log"
STDERR_LOG="$LOG_DIR/stderr.log"

if [ ! -f "$BINARY_PATH" ]; then
    echo "⚫️ Error: zerkalo binary not found at $BINARY_PATH"
    exit 1
fi

if [ ! -x "$OPEN_PATH" ]; then
    echo "⚫️ Error: open was not found at $OPEN_PATH"
    exit 1
fi

mkdir -p "$PLIST_DIR" "$LOG_DIR"
: > "$STDOUT_LOG"
: > "$STDERR_LOG"

cat > "$PLIST_PATH" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>$LABEL</string>

    <key>ProgramArguments</key>
    <array>
        <string>$OPEN_PATH</string>
        <string>-gja</string>
        <string>$APP_DIR</string>
    </array>

    <key>RunAtLoad</key>
    <true/>

    <key>LimitLoadToSessionType</key>
    <array>
        <string>Aqua</string>
    </array>

    <key>WorkingDirectory</key>
    <string>$PROJECT_DIR</string>

    <key>StandardOutPath</key>
    <string>$STDOUT_LOG</string>

    <key>StandardErrorPath</key>
    <string>$STDERR_LOG</string>
</dict>
</plist>
EOF

if ! plutil -lint "$PLIST_PATH" >/dev/null; then
    echo "⚫️ Error: LaunchAgent plist is invalid: $PLIST_PATH"
    exit 1
fi

launchctl bootout "$DOMAIN_TARGET/$LABEL" >/dev/null 2>&1 || true
launchctl bootout "$DOMAIN_TARGET" "$PLIST_PATH" >/dev/null 2>&1 || true
launchctl unload "$PLIST_PATH" >/dev/null 2>&1 || true
pkill -f "$BINARY_PATH" >/dev/null 2>&1 || true

if launchctl bootstrap "$DOMAIN_TARGET" "$PLIST_PATH" >/dev/null 2>&1; then
    launchctl kickstart -k "$DOMAIN_TARGET/$LABEL" >/dev/null 2>&1 || true
elif launchctl load "$PLIST_PATH" >/dev/null 2>&1; then
    :
else
    echo "⚫️ Error: Failed to load the macOS background service automatically."
    echo "⚪️ Try these commands manually:"
    echo "   plutil -lint \"$PLIST_PATH\""
    echo "   launchctl unload \"$PLIST_PATH\" >/dev/null 2>&1 || true"
    echo "   launchctl load \"$PLIST_PATH\""
    exit 1
fi

echo "⚪️ LaunchAgent installed at $PLIST_PATH"
echo "⚪️ Zerkalo will now start automatically at login."
echo "⚪️ The background service now launches the app through LaunchServices for macOS permission support."
echo "⚪️ If permissions are still missing, add the installed app to Accessibility if needed: $APP_DIR"
