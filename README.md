# 🪞 Zerkalo

**Zerkalo** (Зеркало - *Mirror*) is a lightweight macOS daemon written in Rust that provides system-wide real-time phonetic transliteration from Latin to Cyrillic.

## Features

- **System-wide**: Works in all macOS applications (Browsers, IDEs, Messaging apps).
- **Zero Latency**: Direct interception via Quartz Event Tap for maximum responsiveness.
- **Intelligent Phonetic Mapping**: Supports single characters and complex sequences (e.g., `sh` -> `ш`, `shch` -> `щ`).
- **Full Control**: Instantly enable or disable transliteration with a keyboard shortcut.
- **Lightweight**: Minimal CPU and memory usage.

## Installation

1. Ensure you have [Rust](https://rustup.rs/) installed on your Mac.
2. Clone the repository and enter the directory.
3. Run the installation script:
   ```bash
   ./install.sh
   ```
4. The installer compiles Zerkalo, installs a macOS `LaunchAgent`, and configures the optional `zerkalo` shell alias.

## macOS Permissions

To function, **Zerkalo** requires **Accessibility** access to intercept keyboard input:

1. Go to **System Settings** > **Privacy & Security** > **Accessibility**.
2. Add and enable your **Terminal** (or the application that will launch Zerkalo).

## Usage

Zerkalo is designed to run automatically in the background after installation through a macOS `LaunchAgent`.

You can also launch it manually via the run script:
```bash
./run.sh
```

- **Enable / Disable transliteration**: `Cmd` + `Ctrl` + `Z`
- **Quit the daemon**: press `Esc` 5 times in a row
- **Why the hotkey did not start the app before**: a global shortcut only works when a background process is already listening. The installer now sets up that background process automatically.

## Transliteration Table

The mapping follows standard phonetic logic. For more details, see [TRANSLATION_TABLE.md](./TRANSLATION_TABLE.md).

Examples:
- `p` -> `п`
- `r` -> `р`
- `i` -> `и`
- `v` -> `в`
- `e` -> `е`
- `t` -> `т`
- Result: `privet` -> `привет`
