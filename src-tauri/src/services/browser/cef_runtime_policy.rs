use super::{BrowserCapability, BrowserRuntimeHandle};
use std::time::Instant;
#[cfg(native_browser)]
use tauri::Emitter;

#[cfg(native_browser)]
const BROWSER_CAPABILITY_EVENT: &str = "browser-capability-v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CefShutdownBarrier {
    Drained,
    TimedOut,
}

pub(crate) fn begin_cef_shutdown(
    admission_deadline: Instant,
    helper_exit_deadline: Instant,
) -> CefShutdownBarrier {
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    return if super::cef_supervision::emergency::close_gate(
        admission_deadline,
        helper_exit_deadline,
    ) {
        CefShutdownBarrier::Drained
    } else {
        CefShutdownBarrier::TimedOut
    };
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let _ = (admission_deadline, helper_exit_deadline);
        CefShutdownBarrier::Drained
    }
}

pub(crate) fn force_cef_shutdown() {
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    super::cef_supervision::emergency::force_once();
}

pub(crate) fn cef_has_runnable_helpers() -> bool {
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    return super::cef_supervision::emergency::has_runnable();
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    false
}

#[cfg(target_os = "macos")]
pub(super) fn cef_supervision_root() -> std::path::PathBuf {
    crate::services::paths::data_dir().join("cef-supervision")
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(super) fn capability_for_runtime(runtime: &BrowserRuntimeHandle) -> BrowserCapability {
    let runtime_capability = runtime.capability();
    if matches!(runtime_capability, BrowserCapability::Ready { .. })
        && super::session_store::session_key().is_err()
    {
        BrowserCapability::Unavailable {
            restart_recommended: false,
        }
    } else {
        runtime_capability
    }
}

#[cfg(native_browser)]
pub(super) fn emit_capability(app: &tauri::AppHandle, runtime: &BrowserRuntimeHandle) {
    let _ = app.emit(BROWSER_CAPABILITY_EVENT, capability_for_runtime(runtime));
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub(super) fn capability_for_runtime(_runtime: &BrowserRuntimeHandle) -> BrowserCapability {
    BrowserCapability::Hidden
}
