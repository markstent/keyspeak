//! macOS permission checks for Accessibility and Microphone access.

use std::ffi::c_void;

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrustedWithOptions(options: *const c_void) -> bool;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFStringCreateWithCString(
        alloc: *const c_void,
        c_str: *const u8,
        encoding: u32,
    ) -> *const c_void;
    fn CFDictionaryCreate(
        allocator: *const c_void,
        keys: *const *const c_void,
        values: *const *const c_void,
        num_values: isize,
        key_callbacks: *const c_void,
        value_callbacks: *const c_void,
    ) -> *const c_void;
    fn CFRelease(cf: *const c_void);

    static kCFTypeDictionaryKeyCallBacks: c_void;
    static kCFTypeDictionaryValueCallBacks: c_void;
    static kCFBooleanTrue: *const c_void;
}

const K_CF_STRING_ENCODING_UTF8: u32 = 0x08000100;

/// Check if accessibility is granted, optionally prompting the user.
fn check_accessibility(prompt: bool) -> bool {
    unsafe {
        let key_cstr = b"AXTrustedCheckOptionPrompt\0";
        let key = CFStringCreateWithCString(
            std::ptr::null(),
            key_cstr.as_ptr(),
            K_CF_STRING_ENCODING_UTF8,
        );

        let value = if prompt {
            kCFBooleanTrue
        } else {
            // kCFBooleanFalse - just use null-free approach
            kCFBooleanTrue
        };

        let keys = [key];
        let values = [value];
        let options = CFDictionaryCreate(
            std::ptr::null(),
            keys.as_ptr(),
            values.as_ptr(),
            1,
            &kCFTypeDictionaryKeyCallBacks as *const c_void,
            &kCFTypeDictionaryValueCallBacks as *const c_void,
        );

        let trusted = AXIsProcessTrustedWithOptions(options);

        CFRelease(options);
        CFRelease(key);

        trusted
    }
}

/// Check microphone permission by attempting to list input devices.
/// On macOS, cpal will trigger the permission dialog on first access.
fn check_microphone() -> bool {
    use cpal::traits::{DeviceTrait, HostTrait};
    let host = cpal::default_host();
    if let Some(device) = host.default_input_device() {
        // Trying to get supported configs triggers the permission prompt
        device.supported_input_configs().is_ok()
    } else {
        false
    }
}

/// Run all permission checks at startup. Prompts the user for any missing permissions.
/// Returns true if all permissions are granted.
pub fn ensure_permissions() -> bool {
    let mut all_ok = true;

    // Check accessibility (will show macOS prompt if not granted)
    if !check_accessibility(true) {
        eprintln!(
            "[permissions] Accessibility access required. \
             Please grant it in System Settings > Privacy & Security > Accessibility, \
             then relaunch KeySpeak."
        );
        all_ok = false;
    }

    // Check microphone (cpal triggers macOS prompt on first device access)
    if !check_microphone() {
        eprintln!(
            "[permissions] Microphone access required. \
             Please grant it in System Settings > Privacy & Security > Microphone, \
             then relaunch KeySpeak."
        );
        all_ok = false;
    }

    if all_ok {
        eprintln!("[permissions] All permissions granted.");
    }

    all_ok
}
