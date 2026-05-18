#!/bin/bash

# Zerkalo Launch Script

BINARY_PATH="target/release/zerkalo"

if [ -f "$BINARY_PATH" ]; then
    echo "⚪️ Launching Zerkalo..."
    ./$BINARY_PATH
else
    echo "⚫️ Error: Binary not found. Please run ./install.sh first."
    exit 1
fi
