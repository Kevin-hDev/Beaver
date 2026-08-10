use super::super::CefUnavailableCategory;
use super::handle::OwnedHandle;
use super::identity::{open_process, process_exited};
use std::ffi::OsString;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};
use windows_sys::Win32::Foundation::FILETIME;
use windows_sys::Win32::Globalization::{CompareStringOrdinal, CSTR_EQUAL};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows_sys::Win32::System::Threading::{GetProcessTimes, QueryFullProcessImageNameW};

const MAX_PROCESS_PATH_UNITS: usize = 32_768;
const MAX_PROCESS_SCAN_ENTRIES: usize = 65_536;

#[derive(Debug)]
pub(in crate::services::browser) struct WindowsProcessProbe {
    pub(super) parent_pid: u32,
    pub(super) started_at: u64,
    pub(super) executable: PathBuf,
}

impl WindowsProcessProbe {
    pub(in crate::services::browser) fn read(pid: u32) -> Result<Self, CefUnavailableCategory> {
        let handle = open_process(pid)?;
        Self::from_handle(pid, &handle)
    }

    pub(super) fn from_handle(
        pid: u32,
        handle: &OwnedHandle,
    ) -> Result<Self, CefUnavailableCategory> {
        if pid == 0 || process_exited(handle)? {
            return Err(CefUnavailableCategory::Admission);
        }
        Ok(Self {
            parent_pid: parent_pid(pid)?,
            started_at: process_started_at(handle)?,
            executable: process_executable(handle)?,
        })
    }

    pub(in crate::services::browser) fn started_at(&self) -> u64 {
        self.started_at
    }

    pub(in crate::services::browser) fn executable(&self) -> &Path {
        &self.executable
    }
}

fn process_started_at(handle: &OwnedHandle) -> Result<u64, CefUnavailableCategory> {
    let mut creation = FILETIME::default();
    let mut exit = FILETIME::default();
    let mut kernel = FILETIME::default();
    let mut user = FILETIME::default();
    if unsafe {
        GetProcessTimes(
            handle.raw(),
            &mut creation,
            &mut exit,
            &mut kernel,
            &mut user,
        )
    } == 0
    {
        return Err(CefUnavailableCategory::Admission);
    }
    let value = (u64::from(creation.dwHighDateTime) << 32) | u64::from(creation.dwLowDateTime);
    (value != 0)
        .then_some(value)
        .ok_or(CefUnavailableCategory::Admission)
}

fn process_executable(handle: &OwnedHandle) -> Result<PathBuf, CefUnavailableCategory> {
    let mut buffer = Box::new([0_u16; MAX_PROCESS_PATH_UNITS]);
    let mut length = MAX_PROCESS_PATH_UNITS as u32;
    if unsafe { QueryFullProcessImageNameW(handle.raw(), 0, buffer.as_mut_ptr(), &mut length) } == 0
        || length == 0
        || length as usize >= MAX_PROCESS_PATH_UNITS
    {
        return Err(CefUnavailableCategory::Admission);
    }
    let path = PathBuf::from(OsString::from_wide(&buffer[..length as usize]));
    dunce::canonicalize(path).map_err(|_| CefUnavailableCategory::Admission)
}

fn parent_pid(pid: u32) -> Result<u32, CefUnavailableCategory> {
    let snapshot = OwnedHandle::new(unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) })
        .map_err(|_| CefUnavailableCategory::Admission)?;
    let mut entry = PROCESSENTRY32W {
        dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };
    if unsafe { Process32FirstW(snapshot.raw(), &mut entry) } == 0 {
        return Err(CefUnavailableCategory::Admission);
    }
    for _ in 0..MAX_PROCESS_SCAN_ENTRIES {
        if entry.th32ProcessID == pid {
            return (entry.th32ParentProcessID != 0)
                .then_some(entry.th32ParentProcessID)
                .ok_or(CefUnavailableCategory::Admission);
        }
        if unsafe { Process32NextW(snapshot.raw(), &mut entry) } == 0 {
            break;
        }
    }
    Err(CefUnavailableCategory::Admission)
}

pub(super) fn paths_match(actual: &Path, expected: &Path) -> bool {
    let actual = actual.as_os_str().encode_wide().collect::<Vec<_>>();
    let expected = expected.as_os_str().encode_wide().collect::<Vec<_>>();
    if actual.len() > MAX_PROCESS_PATH_UNITS || expected.len() > MAX_PROCESS_PATH_UNITS {
        return false;
    }
    unsafe {
        CompareStringOrdinal(
            actual.as_ptr(),
            actual.len() as i32,
            expected.as_ptr(),
            expected.len() as i32,
            1,
        ) == CSTR_EQUAL
    }
}

pub(super) fn canonical_executable(path: &Path) -> Result<PathBuf, CefUnavailableCategory> {
    if path
        .as_os_str()
        .encode_wide()
        .take(MAX_PROCESS_PATH_UNITS + 1)
        .count()
        > MAX_PROCESS_PATH_UNITS
    {
        return Err(CefUnavailableCategory::Admission);
    }
    let canonical = dunce::canonicalize(path).map_err(|_| CefUnavailableCategory::Admission)?;
    let metadata = canonical
        .metadata()
        .map_err(|_| CefUnavailableCategory::Admission)?;
    let canonical_too_long = canonical
        .as_os_str()
        .encode_wide()
        .take(MAX_PROCESS_PATH_UNITS + 1)
        .count()
        > MAX_PROCESS_PATH_UNITS;
    if !metadata.is_file() || canonical_too_long {
        Err(CefUnavailableCategory::Admission)
    } else {
        Ok(canonical)
    }
}
