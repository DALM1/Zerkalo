mod transliteration;

use std::env;
use std::cell::RefCell;
use std::ffi::c_void;
use std::fs::{File, OpenOptions, create_dir_all};
use std::io::Write;
use std::net::TcpStream;
use std::os::fd::AsRawFd;
use std::path::PathBuf;
use std::process::{self, Command, Stdio};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use core_foundation::runloop::{CFRunLoop, CFRunLoopSource, kCFRunLoopDefaultMode};
use core_graphics::event::{
    CGEvent, CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
    CGEventType, CGEventTapProxy,
};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use foreign_types_shared::ForeignType;
use lazy_static::lazy_static;
use transliteration::{TransliterationAction, TransliterationEngine};

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGEventKeyboardGetUnicodeString(
        event: *const std::ffi::c_void,
        maxStringLength: usize,
        actualStringLength: *mut usize,
        unicodeString: *mut u16,
    );
    fn CGEventKeyboardSetUnicodeString(
        event: *const std::ffi::c_void,
        stringLength: usize,
        unicodeString: *const u16,
    );
    fn CGEventTapPostEvent(proxy: CGEventTapProxy, event: *const std::ffi::c_void);
}

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXIsProcessTrusted() -> bool;
    fn CGPreflightListenEventAccess() -> bool;
    fn CGRequestListenEventAccess() -> bool;
}

#[link(name = "Carbon", kind = "framework")]
unsafe extern "C" {
    fn GetApplicationEventTarget() -> EventTargetRef;
    fn InstallEventHandler(
        target: EventTargetRef,
        handler: EventHandlerProcPtr,
        num_types: u32,
        list: *const EventTypeSpec,
        user_data: *mut c_void,
        out_ref: *mut EventHandlerRef,
    ) -> OSStatus;
    fn RegisterEventHotKey(
        key_code: u32,
        modifiers: u32,
        hot_key_id: EventHotKeyID,
        target: EventTargetRef,
        options: u32,
        out_ref: *mut EventHotKeyRef,
    ) -> OSStatus;
    fn GetEventParameter(
        event: EventRef,
        name: u32,
        desired_type: u32,
        actual_type: *mut u32,
        buffer_size: u32,
        actual_size: *mut u32,
        data: *mut c_void,
    ) -> OSStatus;
    fn RunApplicationEventLoop();
}

type OSStatus = i32;
type EventTargetRef = *mut c_void;
type EventHandlerRef = *mut c_void;
type EventHotKeyRef = *mut c_void;
type EventRef = *mut c_void;
type EventHandlerCallRef = *mut c_void;
type EventHandlerProcPtr = extern "C" fn(EventHandlerCallRef, EventRef, *mut c_void) -> OSStatus;

#[repr(C)]
#[derive(Clone, Copy)]
struct EventTypeSpec {
    event_class: u32,
    event_kind: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct EventHotKeyID {
    signature: u32,
    id: u32,
}

lazy_static! {
    static ref ENGINE: Mutex<TransliterationEngine> = Mutex::new(TransliterationEngine::new());
    static ref LAST_ESC_PRESS: Mutex<Option<Instant>> = Mutex::new(None);
}

thread_local! {
    static WORKER_STATE: RefCell<Option<WorkerHandle>> = const { RefCell::new(None) };
}

static ESC_PRESS_COUNT: AtomicUsize = AtomicUsize::new(0);
static MANAGER_LOCK_FILE: OnceLock<File> = OnceLock::new();

const KEYCODE_ESCAPE: i64 = 53;
const ESC_PRESSES_TO_QUIT: usize = 5;
const ESC_SEQUENCE_TIMEOUT: Duration = Duration::from_secs(2);
const HOTKEY_SIGNATURE: u32 = u32::from_be_bytes(*b"ZRKL");
const HOTKEY_ID_TOGGLE: u32 = 1;
const HOTKEY_ID_QUIT: u32 = 2;
const K_EVENT_CLASS_KEYBOARD: u32 = u32::from_be_bytes(*b"keyb");
const K_EVENT_HOTKEY_PRESSED: u32 = 6;
const K_EVENT_PARAM_DIRECT_OBJECT: u32 = u32::from_be_bytes(*b"----");
const TYPE_EVENT_HOTKEY_ID: u32 = u32::from_be_bytes(*b"hkid");
const OPTION_KEY: u32 = 0x0800;
const DEBUG_ENV_PATH: &str = ".dbg/worker-event-tap.env";
const DEBUG_DEFAULT_URL: &str = "http://127.0.0.1:7777/event";
const DEBUG_DEFAULT_SESSION: &str = "worker-event-tap";
const DEBUG_RUN_ID: &str = "post-fix";

struct WorkerHandle {
    _tap: CGEventTap<'static>,
    loop_source: CFRunLoopSource,
}

fn main() {
    if env::args().any(|arg| arg == "--worker") {
        run_worker();
    } else {
        if !acquire_manager_lock() {
            eprintln!("Zerkalo manager is already running.");
            return;
        }
        run_manager();
    }
}

fn acquire_manager_lock() -> bool {
    let Some(home_dir) = env::var_os("HOME") else {
        return false;
    };

    let mut lock_dir = PathBuf::from(home_dir);
    lock_dir.push("Library/Application Support/Zerkalo");
    if create_dir_all(&lock_dir).is_err() {
        return false;
    }

    let lock_path = lock_dir.join("manager.lock");
    let Ok(lock_file) = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(false)
        .open(lock_path)
    else {
        return false;
    };

    let flock_status = unsafe { libc::flock(lock_file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if flock_status != 0 {
        return false;
    }

    MANAGER_LOCK_FILE.set(lock_file).is_ok()
}

fn run_manager() {
    println!("Zerkalo manager started. Press Alt+Esc to start or stop transliteration.");
    println!("Press Esc 5 times in a row to close Zerkalo. Press Alt+Esc to launch it again.");
    // #region debug-point A:manager-start
    debug_report(
        "A",
        "src/main.rs:run_manager:start",
        "[DEBUG] Manager process started",
        &format!(
            "{{\"pid\":{},\"exe\":{},\"cwd\":{}}}",
            process::id(),
            debug_json_string(
                &env::current_exe()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|error| format!("error:{error}"))
            ),
            debug_json_string(
                &env::current_dir()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|error| format!("error:{error}"))
            ),
        ),
    );
    // #endregion

    if let Err(status) = register_manager_hotkeys() {
        // #region debug-point B:manager-hotkey-register-failure
        debug_report(
            "B",
            "src/main.rs:run_manager:register-failure",
            "[DEBUG] Failed to register manager hotkeys",
            &format!("{{\"status\":{status}}}"),
        );
        // #endregion
        eprintln!("Failed to register manager hotkeys. macOS status: {status}");
        return;
    }
    // #region debug-point A:manager-hotkey-register-success
    debug_report(
        "A",
        "src/main.rs:run_manager:register-success",
        "[DEBUG] Manager hotkeys registered",
        "{}",
    );
    // #endregion

    unsafe {
        RunApplicationEventLoop();
    }
}

fn run_worker() {
    println!("Zerkalo worker started.");
    let tap = match create_worker_tap() {
        Ok(tap) => tap,
        Err(error) => {
            eprintln!("{error}");
            return;
        }
    };

    let loop_source = tap
        .mach_port
        .create_runloop_source(0)
        .expect("Failed to create runloop source");
    let run_loop = CFRunLoop::get_current();
    unsafe {
        run_loop.add_source(&loop_source, kCFRunLoopDefaultMode);
    }
    tap.enable();

    // #region debug-point D:event-loop
    debug_report(
        "D",
        "src/main.rs:run_worker:event-loop",
        "[DEBUG] Worker entering event loop",
        "{}",
    );
    // #endregion

    CFRunLoop::run_current();
}

fn create_worker_tap() -> Result<CGEventTap<'static>, &'static str> {
    // #region debug-point A:worker-start
    debug_report(
        "A",
        "src/main.rs:create_worker_tap:start",
        "[DEBUG] Worker tap creation requested",
        &format!(
            "{{\"pid\":{},\"exe\":{},\"cwd\":{}}}",
            process::id(),
            debug_json_string(
                &env::current_exe()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|error| format!("error:{error}"))
            ),
            debug_json_string(
                &env::current_dir()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|error| format!("error:{error}"))
            ),
        ),
    );
    // #endregion

    // #region debug-point B:permission-state
    let accessibility_trusted = unsafe { AXIsProcessTrusted() };
    let input_monitoring_trusted = unsafe { CGPreflightListenEventAccess() };
    debug_report(
        "B",
        "src/main.rs:create_worker_tap:permission-state",
        "[DEBUG] macOS permission state before event tap creation",
        &format!(
            "{{\"accessibility_trusted\":{},\"input_monitoring_trusted\":{}}}",
            accessibility_trusted, input_monitoring_trusted
        ),
    );
    // #endregion

    if !input_monitoring_trusted {
        let input_monitoring_request_result = unsafe { CGRequestListenEventAccess() };
        // #region debug-point B:input-monitoring-request
        debug_report(
            "B",
            "src/main.rs:create_worker_tap:input-monitoring-request",
            "[DEBUG] Requested macOS Input Monitoring access",
            &format!(
                "{{\"request_result\":{}}}",
                input_monitoring_request_result
            ),
        );
        // #endregion
        return Err(
            "Input Monitoring is not granted for /Users/dalm1/Applications/Zerkalo.app. Enable Zerkalo in System Settings > Privacy & Security > Input Monitoring, then relaunch Zerkalo without reinstalling. If you reinstall, macOS may require you to grant the permission again.",
        );
    }

    if !accessibility_trusted {
        return Err(
            "Accessibility is not granted for /Users/dalm1/Applications/Zerkalo.app. Enable Zerkalo in System Settings > Privacy & Security > Accessibility, then relaunch Zerkalo without reinstalling. If you reinstall, macOS may require you to grant the permission again.",
        );
    }

    // #region debug-point C:hid-attempt
    debug_report(
        "C",
        "src/main.rs:create_worker_tap:hid-attempt",
        "[DEBUG] Attempting HID event tap",
        "{}",
    );
    // #endregion
    let hid_tap = CGEventTap::new(
        CGEventTapLocation::HID,
        CGEventTapPlacement::HeadInsertEventTap,
        CGEventTapOptions::Default,
        vec![CGEventType::KeyDown],
        worker_callback,
    );

    match hid_tap {
        Ok(tap) => {
            // #region debug-point C:hid-success
            debug_report(
                "C",
                "src/main.rs:create_worker_tap:hid-success",
                "[DEBUG] HID event tap created successfully",
                "{}",
            );
            // #endregion
            Ok(tap)
        }
        Err(hid_error) => {
            // #region debug-point C:hid-failure
            debug_report(
                "C",
                "src/main.rs:create_worker_tap:hid-failure",
                "[DEBUG] HID event tap creation failed",
                &format!("{{\"error\":{}}}", debug_json_string(&format!("{hid_error:?}"))),
            );
            // #endregion
            // #region debug-point C:session-attempt
            debug_report(
                "C",
                "src/main.rs:create_worker_tap:session-attempt",
                "[DEBUG] Attempting Session event tap fallback",
                "{}",
            );
            // #endregion
            CGEventTap::new(
                CGEventTapLocation::Session,
                CGEventTapPlacement::HeadInsertEventTap,
                CGEventTapOptions::Default,
                vec![CGEventType::KeyDown],
                worker_callback,
            )
            .map(|tap| {
                // #region debug-point C:session-success
                debug_report(
                    "C",
                    "src/main.rs:create_worker_tap:session-success",
                    "[DEBUG] Session event tap created successfully",
                    "{}",
                );
                // #endregion
                tap
            })
            .map_err(|session_error| {
                // #region debug-point B:session-failure
                debug_report(
                    "B",
                    "src/main.rs:create_worker_tap:session-failure",
                    "[DEBUG] Session event tap creation failed",
                    &format!("{{\"error\":{}}}", debug_json_string(&format!("{session_error:?}"))),
                );
                // #endregion
                // #region debug-point B:worker-tap-failed
                debug_report(
                    "B",
                    "src/main.rs:create_worker_tap:tap-failed",
                    "[DEBUG] Worker failed to create any event tap",
                    "{}",
                );
                // #endregion
                "Failed to create worker event tap. Grant Zerkalo Accessibility and Input Monitoring permissions, then rerun ./install.sh."
            })
        }
    }
}

fn post_backspace(proxy: CGEventTapProxy) {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState).unwrap();
    let down = CGEvent::new_keyboard_event(source.clone(), 51, true).unwrap();
    let up = CGEvent::new_keyboard_event(source, 51, false).unwrap();
    unsafe {
        CGEventTapPostEvent(proxy, down.as_ptr() as *const _);
        CGEventTapPostEvent(proxy, up.as_ptr() as *const _);
    }
}

fn start_worker_on_main_run_loop() -> Result<WorkerHandle, &'static str> {
    // #region debug-point A:spawn-worker
    debug_report(
        "A",
        "src/main.rs:start_worker_on_main_run_loop",
        "[DEBUG] Manager starting worker on main run loop",
        &format!(
            "{{\"manager_pid\":{},\"current_exe\":{}}}",
            process::id(),
            debug_json_string(&env::current_exe().map(|path| path.display().to_string()).unwrap_or_else(|error| format!("error:{error}"))),
        ),
    );
    // #endregion

    let tap = create_worker_tap()?;
    let loop_source = tap
        .mach_port
        .create_runloop_source(0)
        .map_err(|_| "Failed to create runloop source for worker event tap.")?;
    let run_loop = CFRunLoop::get_main();
    unsafe {
        run_loop.add_source(&loop_source, kCFRunLoopDefaultMode);
    }
    tap.enable();
    // #region debug-point D:event-loop-main
    debug_report(
        "D",
        "src/main.rs:start_worker_on_main_run_loop",
        "[DEBUG] Worker tap attached to manager main run loop",
        "{}",
    );
    // #endregion
    Ok(WorkerHandle { _tap: tap, loop_source })
}

fn stop_worker() {
    WORKER_STATE.with(|worker_state| {
        if let Some(worker) = worker_state.borrow_mut().take() {
            let run_loop = CFRunLoop::get_main();
            unsafe {
                run_loop.remove_source(&worker.loop_source, kCFRunLoopDefaultMode);
            }
        }
    });
}

fn toggle_worker() {
    let worker_running = WORKER_STATE.with(|worker_state| worker_state.borrow().is_some());
    // #region debug-point A:toggle-worker
    debug_report(
        "A",
        "src/main.rs:toggle_worker",
        "[DEBUG] Toggle worker requested",
        &format!("{{\"worker_running\":{worker_running}}}"),
    );
    // #endregion

    if worker_running {
        WORKER_STATE.with(|worker_state| {
            if let Some(worker) = worker_state.borrow_mut().take() {
                let run_loop = CFRunLoop::get_main();
                unsafe {
                    run_loop.remove_source(&worker.loop_source, kCFRunLoopDefaultMode);
                }
            }
        });
        println!("Zerkalo DISABLED");
        return;
    }

    match start_worker_on_main_run_loop() {
        Ok(worker) => {
            WORKER_STATE.with(|worker_state| {
                *worker_state.borrow_mut() = Some(worker);
            });
            ENGINE.lock().unwrap().reset();
            speak_phrase("zerkalo");
            println!("Zerkalo ENABLED");
        }
        Err(error) => {
            eprintln!("{error}");
        }
    }
}

fn register_manager_hotkeys() -> Result<(), OSStatus> {
    let event_target = unsafe { GetApplicationEventTarget() };
    if event_target.is_null() {
        return Err(-1);
    }

    let event_spec = EventTypeSpec {
        event_class: K_EVENT_CLASS_KEYBOARD,
        event_kind: K_EVENT_HOTKEY_PRESSED,
    };
    let mut handler_ref: EventHandlerRef = std::ptr::null_mut();
    let install_status = unsafe {
        InstallEventHandler(
            event_target,
            manager_hotkey_handler,
            1,
            &event_spec,
            std::ptr::null_mut(),
            &mut handler_ref,
        )
    };
    if install_status != 0 {
        return Err(install_status);
    }

    let toggle_status = register_hotkey(event_target, HOTKEY_ID_TOGGLE, KEYCODE_ESCAPE as u32, OPTION_KEY);
    if toggle_status != 0 {
        return Err(toggle_status);
    }

    let quit_status = register_hotkey(event_target, HOTKEY_ID_QUIT, KEYCODE_ESCAPE as u32, 0);
    if quit_status != 0 {
        return Err(quit_status);
    }

    Ok(())
}

fn register_hotkey(
    event_target: EventTargetRef,
    id: u32,
    key_code: u32,
    modifiers: u32,
) -> OSStatus {
    let hotkey_id = EventHotKeyID {
        signature: HOTKEY_SIGNATURE,
        id,
    };
    let mut hotkey_ref: EventHotKeyRef = std::ptr::null_mut();
    unsafe { RegisterEventHotKey(key_code, modifiers, hotkey_id, event_target, 0, &mut hotkey_ref) }
}

extern "C" fn manager_hotkey_handler(
    _call_ref: EventHandlerCallRef,
    event: EventRef,
    _user_data: *mut c_void,
) -> OSStatus {
    let mut hotkey_id = EventHotKeyID {
        signature: 0,
        id: 0,
    };
    let status = unsafe {
        GetEventParameter(
            event,
            K_EVENT_PARAM_DIRECT_OBJECT,
            TYPE_EVENT_HOTKEY_ID,
            std::ptr::null_mut(),
            std::mem::size_of::<EventHotKeyID>() as u32,
            std::ptr::null_mut(),
            &mut hotkey_id as *mut EventHotKeyID as *mut c_void,
        )
    };
    if status != 0 || hotkey_id.signature != HOTKEY_SIGNATURE {
        return status;
    }

    // #region debug-point A:hotkey-received
    debug_report(
        "A",
        "src/main.rs:manager_hotkey_handler",
        "[DEBUG] Manager hotkey event received",
        &format!("{{\"hotkey_id\":{}}}", hotkey_id.id),
    );
    // #endregion

    match hotkey_id.id {
        HOTKEY_ID_TOGGLE => {
            ESC_PRESS_COUNT.store(0, Ordering::SeqCst);
            *LAST_ESC_PRESS.lock().unwrap() = None;
            toggle_worker();
        }
        HOTKEY_ID_QUIT => {
            let now = Instant::now();
            let mut last_esc_press = LAST_ESC_PRESS.lock().unwrap();
            let esc_count = if last_esc_press
                .map(|last_press| now.duration_since(last_press) <= ESC_SEQUENCE_TIMEOUT)
                .unwrap_or(false)
            {
                ESC_PRESS_COUNT.fetch_add(1, Ordering::SeqCst) + 1
            } else {
                ESC_PRESS_COUNT.store(1, Ordering::SeqCst);
                1
            };
            *last_esc_press = Some(now);

            if esc_count >= ESC_PRESSES_TO_QUIT {
                stop_worker();
                ESC_PRESS_COUNT.store(0, Ordering::SeqCst);
                *last_esc_press = None;
                println!("Zerkalo closed after 5 Esc presses. Press Alt+Esc to launch it again.");
                speak_phrase("zerkalo closed");
            }
        }
        _ => {}
    }

    0
}

fn worker_callback(proxy: CGEventTapProxy, event_type: CGEventType, event: &CGEvent) -> Option<CGEvent> {
    if matches!(
        event_type,
        CGEventType::TapDisabledByTimeout | CGEventType::TapDisabledByUserInput
    ) {
        // #region debug-point D:tap-reenabled
        debug_report(
            "D",
            "src/main.rs:worker_callback:tap-reenabled",
            "[DEBUG] Worker tap was disabled by macOS and is being re-enabled",
            &format!("{{\"event_type\":{}}}", event_type as u32),
        );
        // #endregion
        WORKER_STATE.with(|worker_state| {
            if let Some(worker) = worker_state.borrow().as_ref() {
                worker._tap.enable();
            }
        });
        return Some(event.clone());
    }

    let mut buffer = [0u16; 4];
    let mut actual_len = 0;

    unsafe {
        CGEventKeyboardGetUnicodeString(
            event.as_ptr() as *const _,
            buffer.len(),
            &mut actual_len,
            buffer.as_mut_ptr(),
        );
    }

    if actual_len > 0 {
        if let Some(c) = std::char::from_u32(buffer[0] as u32) {
            let mut engine = ENGINE.lock().unwrap();
            match engine.process(c) {
                TransliterationAction::Convert(cyr_str) => {
                    let utf16: Vec<u16> = cyr_str.encode_utf16().collect();
                    unsafe {
                        CGEventKeyboardSetUnicodeString(
                            event.as_ptr() as *const _,
                            utf16.len(),
                            utf16.as_ptr(),
                        );
                    }
                }
                TransliterationAction::Replace(backspaces, cyr_str) => {
                    for _ in 0..backspaces {
                        post_backspace(proxy);
                    }
                    let utf16: Vec<u16> = cyr_str.encode_utf16().collect();
                    unsafe {
                        CGEventKeyboardSetUnicodeString(
                            event.as_ptr() as *const _,
                            utf16.len(),
                            utf16.as_ptr(),
                        );
                    }
                }
                TransliterationAction::None => {
                }
            }
        } else {
            ENGINE.lock().unwrap().reset();
        }
    } else {
        ENGINE.lock().unwrap().reset();
    }

    Some(event.clone())
}

fn debug_report(hypothesis_id: &str, location: &str, msg: &str, data_json: &str) {
    let Ok((host, port, path, session_id)) = debug_server_target() else {
        return;
    };
    let Ok(mut stream) = TcpStream::connect((host.as_str(), port)) else {
        return;
    };

    let body = format!(
        "{{\"sessionId\":{},\"runId\":{},\"hypothesisId\":{},\"location\":{},\"msg\":{},\"data\":{},\"ts\":{}}}",
        debug_json_string(&session_id),
        debug_json_string(DEBUG_RUN_ID),
        debug_json_string(hypothesis_id),
        debug_json_string(location),
        debug_json_string(msg),
        data_json,
        debug_now_ms(),
    );
    let request = format!(
        "POST {path} HTTP/1.1\r\nHost: {host}:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len(),
    );
    let _ = stream.write_all(request.as_bytes());
}

fn debug_server_target() -> Result<(String, u16, String, String), ()> {
    let env_content = std::fs::read_to_string(DEBUG_ENV_PATH).unwrap_or_else(|_| {
        format!(
            "DEBUG_SERVER_URL={DEBUG_DEFAULT_URL}\nDEBUG_SESSION_ID={DEBUG_DEFAULT_SESSION}\n"
        )
    });
    let mut url = DEBUG_DEFAULT_URL.to_string();
    let mut session_id = DEBUG_DEFAULT_SESSION.to_string();

    for line in env_content.lines() {
        if let Some(value) = line.strip_prefix("DEBUG_SERVER_URL=") {
            url = value.trim().to_string();
        } else if let Some(value) = line.strip_prefix("DEBUG_SESSION_ID=") {
            session_id = value.trim().to_string();
        }
    }

    let Some(without_scheme) = url.strip_prefix("http://") else {
        return Err(());
    };
    let (host_port, path) = match without_scheme.split_once('/') {
        Some((host_port, rest)) => (host_port, format!("/{}", rest)),
        None => (without_scheme, "/event".to_string()),
    };
    let (host, port_str) = host_port.split_once(':').ok_or(())?;
    let port = port_str.parse::<u16>().map_err(|_| ())?;

    Ok((host.to_string(), port, path, session_id))
}

fn debug_json_string(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r");
    format!("\"{escaped}\"")
}

fn debug_now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
}

fn speak_phrase(phrase: &str) {
    let _ = Command::new("say")
        .arg(phrase)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}
