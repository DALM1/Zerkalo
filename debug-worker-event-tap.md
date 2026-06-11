# Debug Session: worker-event-tap

## Status

- [OPEN] 2026-06-11 Investigation started for worker event tap creation failure on macOS.

## Symptoms

- `Alt+Esc` toggles the manager path.
- Worker process starts, then exits immediately.
- Logs show `Failed to create worker event tap. Do you have Accessibility permissions?`
- User may have granted some permissions already, but current runtime evidence is incomplete.

## Hypotheses

1. The worker process is launched from a binary/path that does not match the app granted in macOS privacy settings, so TCC permission is not applied to the running executable.
2. The worker needs both Accessibility and Input Monitoring, but one of them is still missing for the installed app or for the parent launcher context.
3. The current event tap location/order (`HID` then `Session`) is failing in the `launchd` context, while the same binary might behave differently when started manually.
4. The worker is being spawned correctly, but exits before the event loop becomes active because event tap creation fails for a runtime-specific reason that is not yet logged precisely enough.
5. Old mixed logs from previous manager implementations are obscuring the current failure mode, making the remaining issue look broader than it is.

## Evidence Log

- Instrumentation added in `src/main.rs` to report:
  - manager spawn path,
  - worker `pid` / `exe` / `cwd`,
  - HID attempt,
  - Session fallback attempt,
  - event loop entry.
- `install_launch_agent.sh` log truncation fixed from invalid `:"$LOG"` syntax to `: > "$LOG"`.
- `cargo check` passed after instrumentation.
- `./install.sh` completed successfully and redeployed the instrumented app bundle.
- Direct manual reproduction with the installed binary produced the following runtime evidence:
  - `Worker process started`
  - `Attempting HID event tap`
  - `HID event tap created successfully`
  - `Worker entering event loop`
- This means the installed binary at `/Users/dalm1/Applications/Zerkalo.app/Contents/MacOS/zerkalo` can create the event tap when launched directly from a terminal context.
- Reproduction through the installed manager captured the failing path:
  - `Manager spawning worker`
  - worker `exe` is still `/Users/dalm1/Applications/Zerkalo.app/Contents/MacOS/zerkalo`
  - `Attempting HID event tap`
  - `HID event tap creation failed`
  - `Attempting Session event tap fallback`
  - `Session event tap creation failed`
  - `Worker failed to create any event tap`
- `launchctl print gui/501/com.zerkalo.daemon` confirmed the pre-fix service was running the app binary directly from `launchd`.
- Post-fix installation now configures the LaunchAgent to run `/usr/bin/open -gja /Users/dalm1/Applications/Zerkalo.app`.
- After post-fix install, `launchctl print` shows the LaunchAgent targets `open`, and `pgrep -fal zerkalo` shows the app process is alive.
- Post-fix reproduction still failed, but the new evidence changed the diagnosis:
  - manager PID remained alive,
  - `Manager spawning worker`
  - worker started from the same installed executable,
  - worker `cwd` became `/`,
  - both `HID` and `Session` tap creation still failed.
- This isolates the remaining failure to the child-process worker model itself, not to the executable path and not to the `launchd -> open` launch method.
- A later verification produced an empty debug log and no stdout/stderr output while the app process was still alive.
- `launchctl print gui/501/com.zerkalo.daemon` showed `state = not running`, which is expected now because the LaunchAgent only launches `/usr/bin/open` and then exits.
- The remaining open question is whether `Alt+Esc` reaches the manager hotkey handler after the latest in-process worker refactor.
- Final instrumentation confirmed that `Alt+Esc` is received by the manager and `toggle_worker()` is executed.
- Even in the same process, event tap creation still failed while it was attempted from a secondary thread.
- This isolates the remaining failure to the event tap creation thread context.
- After moving tap creation onto the manager main run loop, event tap creation still failed for both `HID` and `Session`.
- The new strongest hypothesis is now macOS TCC state: the installed app may still not be trusted for Accessibility and/or Input Monitoring even though the same binary once worked when launched from Terminal.
- Final permission instrumentation confirmed the root cause:
  - first runs showed `accessibility_trusted: false`
  - later runs showed `accessibility_trusted: true`
  - failing runs still show `input_monitoring_trusted: false`
- This explains why the global hotkey can still work while Quartz event tap creation keeps failing.

## Hypothesis Status

| ID | Hypothesis | Status | Evidence |
|----|------------|--------|----------|
| A | Wrong executable/path gets launched | Rejected | Both manual and manager-spawn logs show the same installed app binary path. |
| B | Missing macOS permission for installed app | Rejected as primary cause | The same installed binary succeeds when launched directly. |
| C | `launchd` context causes tap creation failure | Rejected after post-fix iteration | Even after switching the LaunchAgent to `open`, the separately spawned worker process still fails to create both event taps. |
| D | Worker exits before event loop for another runtime reason | Rejected | Manual run reaches the event loop; failure happens specifically at event tap creation under the pre-fix launch context. |
| E | Mixed historical logs caused confusion | Confirmed | Fresh logs are now separated and the manager no longer emits the old event tap error path. |

## Root Cause

- macOS accepts event tap creation when Zerkalo runs in the main app process.
- macOS rejects event tap creation when Zerkalo tries to create the transliteration tap outside the main app execution context.
- The evidence now shows two rejected variants:
  - child process worker,
  - secondary thread worker.
- The surviving hypothesis is that the transliteration event tap must be created on the manager main thread and attached to the main run loop.
- That architectural change was completed, but the event tap still fails because macOS does not trust the installed app for the required privacy permissions.

## Minimal Fix

- Changed `install_launch_agent.sh` so the LaunchAgent starts the app through LaunchServices with:
  - `/usr/bin/open`
  - `-gja`
  - `/Users/dalm1/Applications/Zerkalo.app`
- Rationale: launch the same installed app in an app context that matches macOS permission handling instead of executing the binary directly from `launchd`.
- Replaced the `manager -> child worker` model in `src/main.rs` with an in-process worker thread.
- Rationale: keep both the hotkey manager and the transliteration event tap inside the same authorized app process.
- Replaced that secondary-thread worker model with a main-thread event tap attached directly to the manager main run loop.
- Rationale: create the transliteration tap in the exact thread/context that still has working global hotkey access.
- Added explicit instrumentation for `AXIsProcessTrusted()` and `CGPreflightListenEventAccess()` before event tap creation.
- Rationale: distinguish “code/path/thread” problems from a remaining TCC permission denial on the installed app itself.

## Confirmed Root Cause

- The installed app at `/Users/dalm1/Applications/Zerkalo.app` is not currently trusted by macOS for:
- Input Monitoring
- Accessibility was initially false, but later became true after permission changes.
- Because Input Monitoring still remains false, `CGEventTap::new(...)` fails for both `HID` and `Session`.
- The previous successful manual run was consistent with a different permission context, most likely the Terminal app being trusted rather than the installed app bundle.

## Latest Fix

- Added an explicit runtime request for Input Monitoring with `CGRequestListenEventAccess()`.
- Added a precise runtime error message naming `Input Monitoring` and the installed app path.
- Updated installer and docs to stop recommending `./install.sh` after granting permissions.
- Rationale: runtime evidence now shows permissions can still be `false` immediately after a reinstall, so reinstalling is not a safe "reload permissions" step.
- Latest user verification after `open -gja "$HOME/Applications/Zerkalo.app"` still shows:
  - `accessibility_trusted: false`
  - `input_monitoring_trusted: false`
  - `CGRequestListenEventAccess()` returns `false`
- This means macOS still refuses to trust the installed app bundle at runtime, even after relaunch.
- A later verification finally showed the successful path:
  - `accessibility_trusted: true`
  - `input_monitoring_trusted: true`
  - `HID event tap created successfully`
  - `Worker tap attached to manager main run loop`
- After that successful run, the logs show five `hotkey_id: 2` events in a row, matching the `Esc x5` quit path.
- `pgrep -fal zerkalo` is then empty, confirming the manager process has fully exited.
- Therefore `Alt+Esc` cannot relaunch the app after a full quit, because no background process remains to receive the shortcut.

## Next Step

- Re-grant macOS permissions to the installed app bundle itself.
- Reinstall or relaunch once after permissions are enabled.
- Verify again after `Alt+Esc`.
