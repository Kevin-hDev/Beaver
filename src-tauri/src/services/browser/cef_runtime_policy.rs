use super::{BrowserCapability, BrowserRuntimeHandle};

pub(crate) fn begin_cef_shutdown() -> bool {
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    return super::cef_supervision::emergency::close_gate();
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    true
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
        BrowserCapability::Unavailable
    } else {
        runtime_capability
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub(super) fn capability_for_runtime(_runtime: &BrowserRuntimeHandle) -> BrowserCapability {
    BrowserCapability::Hidden
}
