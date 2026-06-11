mod transliteration;

use core_foundation::runloop::{CFRunLoop, kCFRunLoopDefaultMode, CFRunLoopRun};
use foreign_types_shared::ForeignType;
use core_graphics::event::{
    CGEvent, CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions,
    CGEventTapPlacement, CGEventType, CGEventTapProxy,
};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use std::process;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;
use transliteration::{TransliterationEngine, TransliterationAction};
use lazy_static::lazy_static;

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

lazy_static! {
    static ref ENGINE: Mutex<TransliterationEngine> = Mutex::new(TransliterationEngine::new());
}

static ENABLED: AtomicBool = AtomicBool::new(true);
static ESC_PRESS_COUNT: AtomicUsize = AtomicUsize::new(0);

const KEYBOARD_EVENT_KEYCODE_FIELD: u32 = 9;
const KEYCODE_Z: i64 = 6;
const KEYCODE_ESCAPE: i64 = 53;
const ESC_PRESSES_TO_QUIT: usize = 5;

fn main() {
    println!("Zerkalo started. Press Cmd+Ctrl+Z to toggle transliteration.");
    println!("Press Esc 5 times in a row to quit.");

    let current = CGEventTapLocation::HID;
    let tap = match CGEventTap::new(
        current,
        CGEventTapPlacement::HeadInsertEventTap,
        CGEventTapOptions::Default,
        vec![CGEventType::KeyDown],
        callback,
    ) {
        Ok(tap) => tap,
        Err(_) => {
            eprintln!("Failed to create event tap. Do you have Accessibility permissions?");
            return;
        }
    };

    unsafe {
        let loop_source = tap.mach_port.create_runloop_source(0).expect("Failed to create runloop source");
        CFRunLoop::get_main().add_source(&loop_source, kCFRunLoopDefaultMode);
        tap.enable();
        CFRunLoopRun();
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

fn callback(proxy: CGEventTapProxy, _type: CGEventType, event: &CGEvent) -> Option<CGEvent> {
    let flags = event.get_flags();
    let key_code = event.get_integer_value_field(KEYBOARD_EVENT_KEYCODE_FIELD);

    if key_code == KEYCODE_ESCAPE {
        let esc_count = ESC_PRESS_COUNT.fetch_add(1, Ordering::SeqCst) + 1;
        if esc_count >= ESC_PRESSES_TO_QUIT {
            println!("Zerkalo stopped after 5 Esc presses.");
            process::exit(0);
        }
    } else {
        ESC_PRESS_COUNT.store(0, Ordering::SeqCst);
    }

    // Toggle logic: Cmd + Ctrl + Z
    if key_code == KEYCODE_Z
        && flags.contains(CGEventFlags::CGEventFlagCommand | CGEventFlags::CGEventFlagControl)
    {
        let new_state = !ENABLED.load(Ordering::SeqCst);
        ENABLED.store(new_state, Ordering::SeqCst);
        println!("Zerkalo {}", if new_state { "ENABLED" } else { "DISABLED" });
        ENGINE.lock().unwrap().reset();
        return None;
    }

    if !ENABLED.load(Ordering::SeqCst) {
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
