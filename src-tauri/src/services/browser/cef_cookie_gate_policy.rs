#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CefCookieGatePolicy {
    pub(super) verify_store_on_disk: bool,
}

#[cfg(browser_native_api)]
pub(super) fn cef_cookie_gate_policy() -> CefCookieGatePolicy {
    cef_cookie_gate_policy_for_platform(cfg!(target_os = "windows"))
}

pub(super) const fn cef_cookie_gate_policy_for_platform(windows: bool) -> CefCookieGatePolicy {
    CefCookieGatePolicy {
        verify_store_on_disk: !windows,
    }
}
