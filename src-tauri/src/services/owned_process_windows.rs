use super::OwnedProcessError;
use std::sync::OnceLock;
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
#[cfg(test)]
use windows_sys::Win32::System::JobObjects::IsProcessInJob;
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
    SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
};
use windows_sys::Win32::System::Threading::{
    OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SET_QUOTA, PROCESS_TERMINATE,
};

static GLOBAL_JOB: OnceLock<Result<GlobalJob, OwnedProcessError>> = OnceLock::new();

struct GlobalJob(usize);

impl GlobalJob {
    fn new() -> Result<Self, OwnedProcessError> {
        let handle = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
        if handle.is_null() {
            return Err(OwnedProcessError::Admission);
        }
        let job = Self(handle as usize);
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        let configured = unsafe {
            SetInformationJobObject(
                job.raw(),
                JobObjectExtendedLimitInformation,
                std::ptr::addr_of!(limits).cast(),
                std::mem::size_of_val(&limits) as u32,
            )
        };
        if configured == 0 {
            Err(OwnedProcessError::Admission)
        } else {
            Ok(job)
        }
    }

    fn raw(&self) -> HANDLE {
        self.0 as HANDLE
    }
}

impl Drop for GlobalJob {
    fn drop(&mut self) {
        if self.0 != 0 {
            unsafe { CloseHandle(self.raw()) };
        }
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

pub(super) fn release(_pid: u32) {}

#[cfg(test)]
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
