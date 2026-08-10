pub(in crate::services::browser) const CEF_SLOT_CAPACITY: usize = 64;
#[cfg(native_browser)]
pub(crate) const CEF_ADMISSION_SWITCH: &str = "beaver-cef-admission";
pub(super) const CEF_MARKER_MAX_BYTES: usize = 128;
pub(super) const CEF_NONCE_BYTES: usize = 32;
pub(super) const GATE_RECHECK: std::time::Duration = std::time::Duration::from_millis(1);
#[cfg(any(target_os = "windows", target_os = "macos"))]
pub(super) const CEF_TRACKER_POLL: std::time::Duration = std::time::Duration::from_millis(10);
#[cfg(any(windows, target_os = "macos"))]
pub(super) const CEF_ADMISSION_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);
#[cfg(any(windows, target_os = "macos"))]
pub(super) const CEF_HELPER_WAIT_SLICE: std::time::Duration = std::time::Duration::from_millis(10);
