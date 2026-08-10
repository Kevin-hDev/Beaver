use std::time::{Duration, Instant};

#[cfg(target_os = "windows")]
static WINDOWS: std::sync::OnceLock<std::sync::Arc<super::windows::WindowsTrackerShared>> =
    std::sync::OnceLock::new();
#[cfg(target_os = "macos")]
static MACOS: std::sync::OnceLock<std::sync::Arc<super::macos::MacTrackerShared>> =
    std::sync::OnceLock::new();

#[cfg(target_os = "windows")]
pub(super) fn register_windows(
    shared: std::sync::Arc<super::windows::WindowsTrackerShared>,
) -> Result<(), ()> {
    WINDOWS.set(shared).map_err(|_| ())
}

#[cfg(target_os = "macos")]
pub(super) fn register_macos(
    shared: std::sync::Arc<super::macos::MacTrackerShared>,
) -> Result<(), ()> {
    MACOS.set(shared).map_err(|_| ())
}

pub(in crate::services::browser) fn close_gate() -> bool {
    let deadline = Instant::now() + Duration::from_millis(50);
    #[cfg(target_os = "windows")]
    if let Some(shared) = WINDOWS.get() {
        return shared.emergency_close(deadline);
    }
    #[cfg(target_os = "macos")]
    if let Some(shared) = MACOS.get() {
        return shared.emergency_close(deadline);
    }
    true
}

pub(in crate::services::browser) fn force_once() {
    #[cfg(target_os = "windows")]
    if let Some(shared) = WINDOWS.get() {
        shared.emergency_force();
    }
    #[cfg(target_os = "macos")]
    if let Some(shared) = MACOS.get() {
        shared.emergency_force();
    }
}

pub(in crate::services::browser) fn has_runnable() -> bool {
    #[cfg(target_os = "windows")]
    if let Some(shared) = WINDOWS.get() {
        return shared.emergency_has_runnable();
    }
    #[cfg(target_os = "macos")]
    if let Some(shared) = MACOS.get() {
        return shared.emergency_has_runnable();
    }
    false
}
