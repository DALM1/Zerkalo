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
4. The installer compiles Zerkalo, installs a macOS `LaunchAgent` for automatic startup at login, and configures the optional `zerkalo` shell alias.

## macOS Permissions

To function, **Zerkalo** may require both **Accessibility** and **Input Monitoring** access:

1. Go to **System Settings** > **Privacy & Security** > **Accessibility**.
2. Add and enable this installed binary:
   ```text
   ~/Applications/Zerkalo.app
   ```
3. Go to **System Settings** > **Privacy & Security** > **Input Monitoring**.
4. Add and enable the same app there too if macOS prompts for it:
   ```text
   ~/Applications/Zerkalo.app
   ```
5. After granting the permission, relaunch Zerkalo without reinstalling first.
6. Reinstalling can make macOS ask for the permission again for the installed app bundle.
7. If you also run Zerkalo manually from Trae/Terminal, keep your terminal app enabled too.

## Usage

Zerkalo is designed to run automatically in the background after installation through a macOS `LaunchAgent`, and the app manager listens for the global shortcut at all times.

You can also launch it manually via the run script:
```bash
./run.sh
```

- **Start / Stop transliteration**: `Alt` + `Esc`
- **Close and relaunch Zerkalo**: press `Esc` 5 times in a row, then use `Alt` + `Esc` to launch it again
- **Background behavior**: the manager stays loaded in the background so the shortcut can launch the transliteration worker whenever you need it

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
