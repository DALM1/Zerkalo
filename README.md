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

## macOS Permissions

To function, **Zerkalo** requires **Accessibility** access to intercept keyboard input:

1. Go to **System Settings** > **Privacy & Security** > **Accessibility**.
2. Add and enable your **Terminal** (or the application that will launch Zerkalo).

## Usage

Launch the program via the run script:
```bash
./run.sh
```

- **Enable / Disable**: `Cmd` + `Alt` + `C`
- **Quit**: `Ctrl` + `C` in the terminal.

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
