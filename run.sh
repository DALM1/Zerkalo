#!/bin/bash

# Zerkalo Launch Script

APP_DIR="$HOME/Applications/Zerkalo.app"
INSTALLED_BINARY="$APP_DIR/Contents/MacOS/zerkalo"
PROJECT_BINARY="target/release/zerkalo"

if [ -f "$INSTALLED_BINARY" ]; then
    echo "⚪️ Launching Zerkalo..."
    "$INSTALLED_BINARY"
elif [ -f "$PROJECT_BINARY" ]; then
    echo "⚪️ Launching Zerkalo..."
    ./$PROJECT_BINARY
else
    echo "⚫️ Error: Binary not found. Please run ./install.sh first."
    exit 1
fi
