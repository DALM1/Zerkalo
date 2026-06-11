[OPEN] Debug Session: hotkey-not-working

- Symptom: the Zerkalo global shortcut still does not toggle transliteration after installation.
- Expected: the background manager should capture the configured shortcut and start or stop the worker.

## Hypotheses

- A: the LaunchAgent process still fails to create the global event tap under macOS TCC permissions.
- B: the manager event tap is created, but the chosen shortcut is intercepted by macOS or never reaches the callback.
- C: the manager receives the shortcut, but worker startup fails silently.
- D: multiple Zerkalo instances interfere with testing, and the wrong instance is observed.
- E: the LaunchAgent runs in a context that differs from manual terminal launch and loses effective keyboard-monitoring privileges.

## Plan

1. Inspect runtime state of the LaunchAgent and native logs.
2. Confirm which hypotheses still match the current evidence.
3. Add minimal instrumentation only if runtime evidence is insufficient.
4. Apply the smallest possible fix based on confirmed evidence.

## Status

- Evidence collected: LaunchAgent repeatedly failed with `Failed to create manager event tap` when executing the raw binary directly.
- Intermediate fix rejected by evidence: launching through `/usr/bin/open` did not keep the app process alive on this machine.
- Final fix applied:
  - manager shortcut handling now uses native Carbon global hotkeys instead of a Quartz event tap
  - the LaunchAgent runs the installed app binary directly again
  - the manager now enforces a single running instance
- Post-fix evidence:
  - `launchctl print gui/<uid>/com.zerkalo.daemon` shows `state = running` with a live `pid`
  - `pgrep -fal /Users/dalm1/Applications/Zerkalo.app/Contents/MacOS/zerkalo` now shows a single installed manager process
- Waiting for user verification after reinstall.
