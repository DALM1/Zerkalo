# Zerkalo — Real-Time Cyrillic Transliteration Daemon
## Technical Specification

Version: 0.1
Target OS: macOS
Language: Rust
Architecture: Native Quartz Event Tap (low latency)


# 1. Project Goal

Native low-latency macOS daemon capable of intercepting keyboard events globally and transforming latin keystrokes into Cyrillic characters in real-time.

The software :
- work system-wide,
- have near-zero latency,
- avoid fake typing glitches,
- support phonetic transliteration,
- support toggling,
- support intelligent sequence parsing,
- remain lightweight and stable.


# 2. Core Features

## Mandatory Features

- Global keyboard interception
- Real-time transliteration
- Phonetic Russian mapping
- Unicode output
- Toggle hotkey
- Uppercase support
- Low CPU usage
- Works in all applications
- No copy/paste
- No visible input lag

# 3. Technical Architecture

text
Keyboard
   ↓
macOS HID
   ↓
Quartz Event Tap
   ↓
Input Buffer
   ↓
Transliteration Engine
   ↓
Modified Unicode Event
   ↓
Target Application
