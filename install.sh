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
APP_DIR="$HOME/Applications/Zerkalo.app"
APP_CONTENTS_DIR="$APP_DIR/Contents"
APP_MACOS_DIR="$APP_CONTENTS_DIR/MacOS"
APP_RESOURCES_DIR="$APP_CONTENTS_DIR/Resources"
INSTALLED_BINARY="$APP_MACOS_DIR/zerkalo"
ALIAS_SCRIPT="./add_alias.sh"
LAUNCH_AGENT_SCRIPT="./install_launch_agent.sh"

if [ -f "$BINARY_PATH" ]; then
    echo "⚪️ Compilation successful"
    echo ""
    mkdir -p "$APP_MACOS_DIR" "$APP_RESOURCES_DIR"
    cp "$BINARY_PATH" "$INSTALLED_BINARY"
    chmod +x "$INSTALLED_BINARY"
    cat > "$APP_CONTENTS_DIR/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>zerkalo</string>
    <key>CFBundleIdentifier</key>
    <string>com.zerkalo.app</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>Zerkalo</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundleVersion</key>
    <string>0.1.0</string>
    <key>LSUIElement</key>
    <true/>
</dict>
</plist>
EOF
    if command -v codesign >/dev/null 2>&1; then
        if codesign --force --deep --sign - "$APP_DIR" >/dev/null 2>&1; then
            echo "⚪️ Installed app bundle signed for macOS permission stability."
        else
            echo "⚫️ Warning: failed to sign the installed app bundle."
        fi
        echo ""
    fi
    echo "⚪️ Installed app bundle copied to: $APP_DIR"
    echo "⚪️ Installed executable path: $INSTALLED_BINARY"
    echo ""
    if [ -f "$LAUNCH_AGENT_SCRIPT" ]; then
        echo "⚪️ Installing the macOS background service..."
        if "$LAUNCH_AGENT_SCRIPT"; then
            :
        else
            echo "⚫️ Warning: automatic background service installation failed."
            echo "⚪️ You can still use Zerkalo manually with ./run.sh or zerkalo"
        fi
        echo ""
    else
        echo "⚫️ Warning: install_launch_agent.sh was not found, background auto-start was skipped."
        echo ""
    fi
    if [ -f "$ALIAS_SCRIPT" ]; then
        echo "⚪️ Configuring the 'zerkalo' alias..."
        "$ALIAS_SCRIPT"
        echo ""
    else
        echo "⚫️ Warning: add_alias.sh was not found, alias setup was skipped."
        echo ""
    fi
    echo "⚪️ You can then launch the app from a new terminal with: zerkalo"
    echo "⚪️ Zerkalo also starts automatically in the background after login."
    echo "⚪️ The background service launches the installed app through LaunchServices."
    echo ""
    echo "⚪️ IMPORTANT: macOS may require both Accessibility and Input Monitoring permissions."
    echo "1. Go to System Settings > Privacy & Security > Accessibility."
    echo "2. Add and enable this exact app if needed: $APP_DIR"
    echo "3. Go to System Settings > Privacy & Security > Input Monitoring."
    echo "4. Add and enable this exact app there too if macOS prompts for it: $APP_DIR"
    echo "5. After granting permission, relaunch Zerkalo without reinstalling first."
    echo "6. Reinstalling can make macOS ask for the permission again for this app bundle."
    echo "7. If you run it manually from Trae/Terminal, also keep your terminal app enabled."
    echo ""
    echo "⚪️ Shortcut: Alt + Esc to enable/disable transliteration."
    echo "⚪️ Close Zerkalo by pressing Esc 5 times in a row, then launch it again with Alt + Esc."
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
