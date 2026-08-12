use super::OwnedProcessError;
use std::os::windows::io::{AsHandle, AsRawHandle};
use std::sync::OnceLock;
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0};
use windows_sys::Win32::System::JobObjects::{AssignProcessToJobObject, IsProcessInJob};
use windows_sys::Win32::System::Threading::{
    OpenProcess, TerminateProcess, WaitForSingleObject, PROCESS_QUERY_LIMITED_INFORMATION,
    PROCESS_SET_QUOTA, PROCESS_TERMINATE,
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
