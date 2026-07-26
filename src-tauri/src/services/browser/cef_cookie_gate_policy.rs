#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CefCookieGatePolicy {
    pub(super) verify_store_on_disk: bool,
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(super) fn cef_cookie_gate_policy() -> CefCookieGatePolicy {
    cef_cookie_gate_policy_for_platform(cfg!(target_os = "windows"))
}

pub(super) const fn cef_cookie_gate_policy_for_platform(windows: bool) -> CefCookieGatePolicy {
    CefCookieGatePolicy {
        verify_store_on_disk: !windows,
    }
}
