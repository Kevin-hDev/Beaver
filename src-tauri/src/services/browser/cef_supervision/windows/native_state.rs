use super::native_authority::WindowsTerminationState;
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};

pub(super) const NATIVE_FREE: u8 = 0;
pub(super) const NATIVE_WRITING: u8 = 1;
pub(super) const NATIVE_PREPARED: u8 = 2;
pub(super) const NATIVE_ADMITTED: u8 = 3;
pub(super) const NATIVE_INSPECTING: u8 = 4;
pub(super) const NATIVE_TERMINATING: u8 = 5;
pub(super) const NATIVE_EXITED: u8 = 6;
pub(super) const NATIVE_CLEANING: u8 = 7;

pub(super) fn state_for(state: WindowsTerminationState) -> u8 {
    match state {
        WindowsTerminationState::Admitted => NATIVE_ADMITTED,
        WindowsTerminationState::Terminating => NATIVE_TERMINATING,
        WindowsTerminationState::Exited => NATIVE_EXITED,
    }
}

pub(super) fn close(handle: HANDLE) {
    if !handle.is_null() {
        unsafe { CloseHandle(handle) };
    }
}
