use std::sync::{Arc, Mutex};

use tauri::Manager;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WindowSignal {
    FocusChanged(bool),
    Hidden,
    Minimized,
}

#[derive(Clone, Default)]
pub struct WindowEventState(Arc<Mutex<Option<WindowSignal>>>);

impl WindowEventState {
    pub fn record(&self, signal: WindowSignal) {
        if let Ok(mut current) = self.0.lock() {
            let stop_is_pending = matches!(
                *current,
                Some(WindowSignal::Hidden | WindowSignal::Minimized)
            );
            if !stop_is_pending || !matches!(signal, WindowSignal::FocusChanged(_)) {
                *current = Some(signal);
            }
        }
    }

    pub fn take_stop_signal(&self) -> Option<WindowSignal> {
        let mut current = self.0.lock().ok()?;
        match *current {
            Some(WindowSignal::Hidden | WindowSignal::Minimized) => current.take(),
            _ => None,
        }
    }
}

pub fn record_for_app(app: &tauri::AppHandle, signal: WindowSignal) {
    if let Some(state) = app.try_state::<WindowEventState>() {
        state.record(signal);
    }
}

#[cfg(target_os = "macos")]
pub fn session_is_locked() -> bool {
    use core_foundation::{
        base::{CFType, TCFType},
        boolean::CFBoolean,
        dictionary::{CFDictionary, CFDictionaryRef},
        string::CFString,
    };

    #[link(name = "ApplicationServices", kind = "framework")]
    unsafe extern "C" {
        fn CGSessionCopyCurrentDictionary() -> CFDictionaryRef;
    }

    let raw = unsafe { CGSessionCopyCurrentDictionary() };
    if raw.is_null() {
        return false;
    }
    let dictionary: CFDictionary<CFString, CFType> =
        unsafe { TCFType::wrap_under_create_rule(raw) };
    dictionary
        .find(CFString::new("CGSSessionScreenIsLocked"))
        .and_then(|value| value.downcast::<CFBoolean>())
        .map(bool::from)
        .unwrap_or(false)
}

#[cfg(windows)]
pub fn session_is_locked() -> bool {
    use windows_sys::Win32::System::StationsAndDesktops::{
        CloseDesktop, OpenInputDesktop, DESKTOP_READOBJECTS,
    };

    let desktop = unsafe { OpenInputDesktop(0, 0, DESKTOP_READOBJECTS) };
    if desktop.is_null() {
        return true;
    }
    unsafe {
        CloseDesktop(desktop);
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focus_loss_alone_never_stops_capture() {
        let state = WindowEventState::default();
        state.record(WindowSignal::FocusChanged(false));
        assert_eq!(state.take_stop_signal(), None);
        state.record(WindowSignal::Hidden);
        state.record(WindowSignal::FocusChanged(false));
        assert_eq!(state.take_stop_signal(), Some(WindowSignal::Hidden));
    }
}
