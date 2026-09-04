#[cfg(target_os = "macos")]
pub(super) fn shift_pressed() -> Option<bool> {
    use objc2_app_kit::{NSEvent, NSEventModifierFlags};

    let flags = NSEvent::modifierFlags_class();
    Some(flags.contains(NSEventModifierFlags::Shift))
}

#[cfg(target_os = "windows")]
pub(super) fn shift_pressed() -> Option<bool> {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_SHIFT};

    // SAFETY: GetAsyncKeyState reads process-independent keyboard state and
    // receives the fixed VK_SHIFT virtual key, with no borrowed pointers.
    Some(unsafe { GetAsyncKeyState(VK_SHIFT as i32) } < 0)
}

#[cfg(target_os = "linux")]
pub(super) fn shift_pressed() -> Option<bool> {
    // SAFETY: every symbol comes from one live libX11 handle, all arguments are
    // fixed or owned locally, and the display is closed before the handle.
    unsafe { x11_shift_pressed() }
}

#[cfg(target_os = "linux")]
unsafe fn x11_shift_pressed() -> Option<bool> {
    use std::ffi::{c_char, c_int, c_ulong, c_void};

    type Open = unsafe extern "C" fn(*const c_char) -> *mut c_void;
    type Query = unsafe extern "C" fn(*mut c_void, *mut c_char) -> c_int;
    type Keycode = unsafe extern "C" fn(*mut c_void, c_ulong) -> u8;
    type Close = unsafe extern "C" fn(*mut c_void) -> c_int;

    let library =
        unsafe { libc::dlopen(c"libX11.so.6".as_ptr(), libc::RTLD_NOW | libc::RTLD_LOCAL) };
    if library.is_null() {
        return None;
    }
    let open: Open = unsafe { load_symbol(library, c"XOpenDisplay".as_ptr())? };
    let query: Query = unsafe { load_symbol(library, c"XQueryKeymap".as_ptr())? };
    let keycode: Keycode = unsafe { load_symbol(library, c"XKeysymToKeycode".as_ptr())? };
    let close: Close = unsafe { load_symbol(library, c"XCloseDisplay".as_ptr())? };
    let display = unsafe { open(std::ptr::null()) };
    if display.is_null() {
        unsafe { libc::dlclose(library) };
        return None;
    }
    let mut keys = [0_i8; 32];
    let queried = unsafe { query(display, keys.as_mut_ptr()) } != 0;
    let left = unsafe { keycode(display, 0xffe1) };
    let right = unsafe { keycode(display, 0xffe2) };
    let pressed = queried && (key_pressed(&keys, left) || key_pressed(&keys, right));
    unsafe {
        close(display);
        libc::dlclose(library);
    }
    Some(pressed)
}

#[cfg(target_os = "linux")]
unsafe fn load_symbol<T: Copy>(
    library: *mut std::ffi::c_void,
    name: *const std::ffi::c_char,
) -> Option<T> {
    let symbol = unsafe { libc::dlsym(library, name) };
    if symbol.is_null() {
        unsafe { libc::dlclose(library) };
        return None;
    }
    Some(unsafe { std::mem::transmute_copy(&symbol) })
}

#[cfg(target_os = "linux")]
fn key_pressed(keys: &[i8; 32], keycode: u8) -> bool {
    keycode != 0 && (keys[(keycode / 8) as usize] as u8 & (1 << (keycode % 8))) != 0
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
pub(super) fn shift_pressed() -> Option<bool> {
    None
}
