use super::super::CefUnavailableCategory;
use super::handle::OwnedHandle;
use super::process_query::WindowsProcessProbe;
use std::path::Path;
use windows_sys::Win32::Foundation::{WAIT_OBJECT_0, WAIT_TIMEOUT};
use windows_sys::Win32::System::Threading::{OpenProcess, WaitForSingleObject};

const SYNCHRONIZE_ACCESS: u32 = 0x0010_0000;
const PROCESS_TERMINATE_ACCESS: u32 = 0x0000_0001;
const PROCESS_SET_QUOTA_ACCESS: u32 = 0x0000_0100;
const PROCESS_QUERY_LIMITED_ACCESS: u32 = 0x0000_1000;

pub(in crate::services::browser) const CEF_PROCESS_ACCESS_RIGHTS: u32 = SYNCHRONIZE_ACCESS
    | PROCESS_TERMINATE_ACCESS
    | PROCESS_SET_QUOTA_ACCESS
    | PROCESS_QUERY_LIMITED_ACCESS;

pub(in crate::services::browser) struct WindowsProcessIdentity {
    handle: OwnedHandle,
    pid: u32,
    parent_pid: u32,
    started_at: u64,
}

impl WindowsProcessIdentity {
    pub(in crate::services::browser) fn acquire(
        pid: u32,
        expected_parent: u32,
        expected_started_at: u64,
        expected_executable: &Path,
    ) -> Result<Self, CefUnavailableCategory> {
        let handle = open_process(pid)?;
        let probe = WindowsProcessProbe::from_handle(pid, &handle)?;
        let expected = super::process_query::canonical_executable(expected_executable)?;
        if expected_parent == 0
            || expected_started_at == 0
            || probe.parent_pid != expected_parent
            || probe.started_at != expected_started_at
            || !super::process_query::paths_match(&probe.executable, &expected)
        {
            return Err(CefUnavailableCategory::Admission);
        }
        Ok(Self {
            handle,
            pid,
            parent_pid: probe.parent_pid,
            started_at: probe.started_at,
        })
    }

    pub(in crate::services::browser) fn pid(&self) -> u32 {
        self.pid
    }

    pub(in crate::services::browser) fn parent_pid(&self) -> u32 {
        self.parent_pid
    }

    pub(in crate::services::browser) fn started_at(&self) -> u64 {
        self.started_at
    }

    pub(in crate::services::browser) fn is_exited(&self) -> Result<bool, CefUnavailableCategory> {
        process_exited(&self.handle)
    }

    pub(in crate::services::browser) fn wait_for_exit(
        &self,
        timeout_ms: u32,
    ) -> Result<bool, CefUnavailableCategory> {
        match unsafe { WaitForSingleObject(self.handle.raw(), timeout_ms) } {
            WAIT_OBJECT_0 => Ok(true),
            WAIT_TIMEOUT => Ok(false),
            _ => Err(CefUnavailableCategory::Admission),
        }
    }

    pub(super) fn raw_handle(&self) -> windows_sys::Win32::Foundation::HANDLE {
        self.handle.raw()
    }

    pub(super) fn terminate(&self) -> Result<(), CefUnavailableCategory> {
        if unsafe { windows_sys::Win32::System::Threading::TerminateProcess(self.handle.raw(), 1) }
            == 0
        {
            Err(CefUnavailableCategory::Admission)
        } else {
            Ok(())
        }
    }

    pub(super) fn into_parts(self) -> WindowsProcessParts {
        WindowsProcessParts {
            handle: self.handle.into_raw(),
            pid: self.pid,
            parent_pid: self.parent_pid,
            started_at: self.started_at,
        }
    }
}

impl std::fmt::Debug for WindowsProcessIdentity {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("WindowsProcessIdentity([redacted])")
    }
}

pub(super) struct WindowsProcessParts {
    pub(super) handle: windows_sys::Win32::Foundation::HANDLE,
    pub(super) pid: u32,
    pub(super) parent_pid: u32,
    pub(super) started_at: u64,
}

pub(super) fn open_process(pid: u32) -> Result<OwnedHandle, CefUnavailableCategory> {
    if pid == 0 {
        return Err(CefUnavailableCategory::Admission);
    }
    OwnedHandle::new(unsafe { OpenProcess(CEF_PROCESS_ACCESS_RIGHTS, 0, pid) })
        .map_err(|_| CefUnavailableCategory::Admission)
}

pub(super) fn process_exited(handle: &OwnedHandle) -> Result<bool, CefUnavailableCategory> {
    match unsafe { WaitForSingleObject(handle.raw(), 0) } {
        WAIT_OBJECT_0 => Ok(true),
        WAIT_TIMEOUT => Ok(false),
        _ => Err(CefUnavailableCategory::Admission),
    }
}
