use super::{OwnedProcessError, OwnedProcessIdentity};
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::os::windows::fs::OpenOptionsExt;
use std::os::windows::io::{AsHandle, AsRawHandle};
use std::path::PathBuf;
use std::sync::OnceLock;
use windows_sys::Win32::Foundation::{CloseHandle, FILETIME, HANDLE, WAIT_OBJECT_0};
use windows_sys::Win32::Storage::FileSystem::{
    GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION, FILE_FLAG_OPEN_REPARSE_POINT,
};
use windows_sys::Win32::System::JobObjects::{AssignProcessToJobObject, IsProcessInJob};
use windows_sys::Win32::System::Threading::{
    GetProcessTimes, OpenProcess, QueryFullProcessImageNameW, TerminateProcess,
    WaitForSingleObject, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SET_QUOTA, PROCESS_TERMINATE,
};

static GLOBAL_JOB: OnceLock<Result<GlobalJob, OwnedProcessError>> = OnceLock::new();

struct GlobalJob(windows_spawn::Job);

impl GlobalJob {
    fn new() -> Result<Self, OwnedProcessError> {
        let job = windows_spawn::Job::create().map_err(|_| OwnedProcessError::Admission)?;
        job.set_kill_on_close(true)
            .map_err(|_| OwnedProcessError::Admission)?;
        Ok(Self(job))
    }

    fn raw(&self) -> HANDLE {
        self.0.as_handle().as_raw_handle()
    }
}

struct ProcessHandle(HANDLE);

impl ProcessHandle {
    fn open(pid: u32, access: u32) -> Result<Self, OwnedProcessError> {
        let handle = unsafe { OpenProcess(access, 0, pid) };
        if handle.is_null() {
            Err(OwnedProcessError::Admission)
        } else {
            Ok(Self(handle))
        }
    }
}

impl Drop for ProcessHandle {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { CloseHandle(self.0) };
        }
    }
}

fn job() -> Result<&'static GlobalJob, OwnedProcessError> {
    GLOBAL_JOB
        .get_or_init(GlobalJob::new)
        .as_ref()
        .map_err(|error| *error)
}

pub(super) fn admit(pid: u32) -> Result<(), OwnedProcessError> {
    if pid < 2 {
        return Err(OwnedProcessError::Admission);
    }
    let process = ProcessHandle::open(
        pid,
        PROCESS_SET_QUOTA | PROCESS_TERMINATE | PROCESS_QUERY_LIMITED_INFORMATION,
    )?;
    if unsafe { AssignProcessToJobObject(job()?.raw(), process.0) } == 0 {
        Err(OwnedProcessError::Admission)
    } else {
        Ok(())
    }
}

pub(super) fn admit_suspended_handle(process: HANDLE) -> Result<(), OwnedProcessError> {
    if unsafe { AssignProcessToJobObject(job()?.raw(), process) } == 0 {
        Err(OwnedProcessError::Admission)
    } else {
        Ok(())
    }
}

pub(super) fn identity(pid: u32) -> Result<OwnedProcessIdentity, OwnedProcessError> {
    if pid < 2 {
        return Err(OwnedProcessError::Admission);
    }
    let process = ProcessHandle::open(pid, PROCESS_QUERY_LIMITED_INFORMATION)?;
    let native_start_time = start_time(process.0)?;
    let mut contained = 0;
    let in_job =
        unsafe { IsProcessInJob(process.0, job()?.raw(), &mut contained) } != 0 && contained != 0;
    if !in_job {
        return Err(OwnedProcessError::Admission);
    }
    Ok(OwnedProcessIdentity {
        pid,
        native_scope: 1,
        native_start_time,
        executable: executable_identity(process.0)?,
    })
}

pub(super) fn identity_matches(expected: OwnedProcessIdentity) -> Result<(), OwnedProcessError> {
    (identity(expected.pid)? == expected)
        .then_some(())
        .ok_or(OwnedProcessError::Admission)
}

pub(super) fn global_job_scope() -> u64 {
    1
}

pub(super) fn matches_native_start(pid: u32, expected: u64) -> bool {
    identity(pid).is_ok_and(|identity| identity.native_start_time == expected)
}

pub(super) fn process_exists(pid: u32) -> bool {
    ProcessHandle::open(pid, PROCESS_QUERY_LIMITED_INFORMATION).is_ok()
}

pub(super) fn terminate_native(pid: u32) {
    if let Ok(process) = ProcessHandle::open(pid, PROCESS_TERMINATE) {
        unsafe { TerminateProcess(process.0, 1) };
    }
}

pub(super) fn spawn_conpty<T: windows_spawn::AsPseudoConsole>(
    command: &mut windows_spawn::Command,
    pseudoconsole: &T,
) -> Result<windows_spawn::Child, OwnedProcessError> {
    let options = windows_spawn::SpawnOptions::new()
        .job(&job()?.0)
        .pseudoconsole(pseudoconsole);
    command
        .spawn_with(options)
        .map_err(|error| OwnedProcessError::Spawn(error.kind()))
}

pub(super) fn release(_pid: u32) {}

pub(super) fn is_confined(pid: u32) -> bool {
    let Ok(job) = job() else {
        return false;
    };
    let Ok(process) = ProcessHandle::open(pid, PROCESS_QUERY_LIMITED_INFORMATION) else {
        return false;
    };
    let mut contained = 0;
    (unsafe { IsProcessInJob(process.0, job.raw(), &mut contained) }) != 0 && contained != 0
}

pub(super) fn terminate_confined(pids: &[u32], deadline: std::time::Instant) -> usize {
    let Ok(job) = job() else {
        return 0;
    };
    let processes = pids
        .iter()
        .filter_map(|pid| {
            let process =
                ProcessHandle::open(*pid, PROCESS_TERMINATE | PROCESS_QUERY_LIMITED_INFORMATION)
                    .ok()?;
            let mut contained = 0;
            let in_job = unsafe { IsProcessInJob(process.0, job.raw(), &mut contained) } != 0
                && contained != 0;
            in_job.then_some(process)
        })
        .collect::<Vec<_>>();

    // Every ownership check and termination share one retained process handle,
    // so PID reuse cannot cross the boundary between the two phases.
    for process in &processes {
        // A concurrent natural exit may make TerminateProcess report access denied;
        // the following wait remains the authority for whether teardown completed.
        unsafe { TerminateProcess(process.0, 1) };
    }
    processes
        .iter()
        .filter(|process| {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            let timeout_ms = remaining.as_millis().min(u128::from(u32::MAX)) as u32;
            unsafe { WaitForSingleObject(process.0, timeout_ms) == WAIT_OBJECT_0 }
        })
        .count()
}

fn start_time(process: HANDLE) -> Result<u64, OwnedProcessError> {
    let mut creation = FILETIME::default();
    let mut exit = FILETIME::default();
    let mut kernel = FILETIME::default();
    let mut user = FILETIME::default();
    if unsafe { GetProcessTimes(process, &mut creation, &mut exit, &mut kernel, &mut user) } == 0 {
        return Err(OwnedProcessError::Admission);
    }
    Some((u64::from(creation.dwHighDateTime) << 32) | u64::from(creation.dwLowDateTime))
        .filter(|value| *value > 0)
        .ok_or(OwnedProcessError::Admission)
}

fn executable_identity(process: HANDLE) -> Result<u128, OwnedProcessError> {
    const MAX_PATH_UNITS: usize = 32_768;
    let mut buffer = vec![0_u16; MAX_PATH_UNITS];
    let mut length = buffer.len() as u32;
    if unsafe { QueryFullProcessImageNameW(process, 0, buffer.as_mut_ptr(), &mut length) } == 0
        || length == 0
        || length as usize >= MAX_PATH_UNITS
    {
        return Err(OwnedProcessError::Admission);
    }
    let path = PathBuf::from(OsString::from_wide(&buffer[..length as usize]));
    let file = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)
        .map_err(|_| OwnedProcessError::Admission)?;
    let mut info = std::mem::MaybeUninit::<BY_HANDLE_FILE_INFORMATION>::zeroed();
    if unsafe { GetFileInformationByHandle(file.as_raw_handle() as _, info.as_mut_ptr()) } == 0 {
        return Err(OwnedProcessError::Admission);
    }
    let info = unsafe { info.assume_init() };
    let file_id = (u64::from(info.nFileIndexHigh) << 32) | u64::from(info.nFileIndexLow);
    let value = (u128::from(info.dwVolumeSerialNumber) << 64) | u128::from(file_id);
    (value != 0)
        .then_some(value)
        .ok_or(OwnedProcessError::Admission)
}
