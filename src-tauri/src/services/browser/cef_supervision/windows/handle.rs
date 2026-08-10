use super::super::CefUnavailableCategory;
use std::fmt;
use windows_sys::Win32::Foundation::{CloseHandle, GetHandleInformation, HANDLE};

pub(super) struct OwnedHandle(HANDLE);

impl OwnedHandle {
    pub(super) fn new(handle: HANDLE) -> Result<Self, CefUnavailableCategory> {
        if handle.is_null() {
            Err(CefUnavailableCategory::Object)
        } else {
            Ok(Self(handle))
        }
    }

    pub(super) fn raw(&self) -> HANDLE {
        self.0
    }

    pub(super) fn is_non_inheritable(&self) -> bool {
        let mut flags = 0_u32;
        (unsafe { GetHandleInformation(self.0, &mut flags) }) != 0 && flags & 1 == 0
    }
}

impl fmt::Debug for OwnedHandle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("OwnedHandle([redacted])")
    }
}

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { CloseHandle(self.0) };
        }
    }
}
