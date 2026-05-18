#!/bin/bash

# Zerkalo Installation Script
# This script compiles the project and provides instructions for permissions.

set -e

echo "⚪️ Initializing Zerkalo..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null
then
    echo "⚫️ Error: Rust/Cargo is not installed. Please install it via https://rustup.rs/"
    exit 1
fi

echo "⚪️ Compiling project in release mode..."
cargo build --release

BINARY_PATH="target/release/zerkalo"

if [ -f "$BINARY_PATH" ]; then
    echo "⚪️ Compilation successful"
    echo ""
    echo "⚪️ IMPORTANT: macOS requires special permissions to intercept keyboard input."
    echo "1. Go to System Settings > Privacy & Security > Accessibility."
    echo "2. Add and enable your Terminal (or the application that will launch Zerkalo)."
    echo ""
    echo "⚪️ Shortcut: Cmd + Alt + C to enable/disable transliteration."
    echo ""

    read -p "⚪️ Would you like to launch Zerkalo now? (y/n) " choice
    case "$choice" in
      y|Y )
        ./run.sh
        ;;
      * )
        echo "⚪️ Installation complete. You can launch the program later with ./run.sh"
        ;;
    esac
else
    echo "⚫️ Error: Compilation failed."
    exit 1
fi
