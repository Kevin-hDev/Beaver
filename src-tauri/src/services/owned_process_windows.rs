use super::{OwnedProcessError, OwnedProcessIdentity, OwnedProcessInspection};
#[path = "owned_process_windows_recovery.rs"]
mod recovery;
#[path = "owned_process_windows_termination.rs"]
mod termination;
pub(super) use termination::recover_exact_with_cancel;
#[path = "owned_process_windows_support.rs"]
mod support;
use std::os::windows::io::{AsHandle, AsRawHandle};
use std::sync::OnceLock;
pub(super) use support::DedicatedJob;
use windows_sys::Win32::Foundation::{CloseHandle, FILETIME, HANDLE, STILL_ACTIVE};
use windows_sys::Win32::System::JobObjects::{AssignProcessToJobObject, IsProcessInJob};
use windows_sys::Win32::System::Threading::{
    GetExitCodeProcess, GetProcessId, GetProcessTimes, OpenProcess, QueryFullProcessImageNameW,
    TerminateProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SET_QUOTA, PROCESS_TERMINATE,
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
    identity_from_handle(process.0)
}

pub(super) fn inspect_for_recovery(pid: u32) -> Result<OwnedProcessInspection, OwnedProcessError> {
    recovery::inspect_for_recovery(pid)
}

pub(super) fn identity_from_handle(
    process: HANDLE,
) -> Result<OwnedProcessIdentity, OwnedProcessError> {
    let executable = executable_identity(process)?;
    identity_from_handle_observed(process, executable)
}

pub(super) fn identity_from_handle_with_executable(
    process: HANDLE,
    expected_executable: u128,
) -> Result<OwnedProcessIdentity, OwnedProcessError> {
    let executable = executable_identity(process)?;
    if expected_executable == 0 || executable != expected_executable {
        return Err(OwnedProcessError::Admission);
    }
    identity_from_handle_observed(process, executable)
}

fn identity_from_handle_observed(
    process: HANDLE,
    executable: u128,
) -> Result<OwnedProcessIdentity, OwnedProcessError> {
    let pid = unsafe { GetProcessId(process) };
    if pid < 2 {
        return Err(OwnedProcessError::Admission);
    }
    let native_start_time = start_time(process)?;
    if !is_in_owned_job(process)? {
        return Err(OwnedProcessError::Admission);
    }
    Ok(OwnedProcessIdentity {
        pid,
        native_scope: 1,
        native_start_time,
        executable,
    })
}

fn is_in_owned_job(process: HANDLE) -> Result<bool, OwnedProcessError> {
    let mut contained = 0;
    if unsafe { IsProcessInJob(process, job()?.raw(), &mut contained) } == 0 {
        return Err(OwnedProcessError::Admission);
    }
    Ok(contained != 0)
}

pub(super) fn signal_exact(
    expected: OwnedProcessIdentity,
    force: bool,
) -> Result<(), OwnedProcessError> {
    let process = ProcessHandle::open(
        expected.pid,
        PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_TERMINATE,
    )?;
    if identity_from_handle_with_executable(process.0, expected.executable)? != expected {
        return Err(OwnedProcessError::Admission);
    }
    (unsafe { TerminateProcess(process.0, if force { 1 } else { 0 }) } != 0)
        .then_some(())
        .ok_or(OwnedProcessError::Admission)
}

pub(super) fn process_exists(pid: u32) -> bool {
    let Ok(process) = ProcessHandle::open(pid, PROCESS_QUERY_LIMITED_INFORMATION) else {
        return false;
    };
    let mut exit_code = 0_u32;
    // Windows can keep an exited process object open while handles still
    // exist. Callers need liveness, not merely the ability to open that PID.
    (unsafe { GetExitCodeProcess(process.0, &mut exit_code) }) != 0
        && exit_code == STILL_ACTIVE as u32
}

pub(super) fn reap_exited_child(_pid: u32) -> Result<bool, OwnedProcessError> {
    Ok(false)
}

pub(super) fn spawn_conpty<T: windows_spawn::AsPseudoConsole>(
    command: &mut windows_spawn::Command,
    pseudoconsole: &T,
) -> Result<windows_spawn::Child, OwnedProcessError> {
    support::spawn_conpty(command, pseudoconsole)
}

pub(super) fn release(pid: u32) {
    support::release(pid);
}

pub(super) fn is_confined(pid: u32) -> bool {
    support::is_confined(pid)
}

pub(super) fn terminate_confined(pids: &[u32], deadline: std::time::Instant) -> usize {
    support::terminate_confined(pids, deadline)
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
    crate::services::ollama_manager::windows_image_identity_from_path(&buffer[..length as usize])
        .ok_or(OwnedProcessError::Admission)
}
