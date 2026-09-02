use super::OwnedProcessError;
use super::{job, ProcessHandle};
use windows_sys::Win32::Foundation::WAIT_OBJECT_0;
use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Thread32First, Thread32Next, TH32CS_SNAPTHREAD, THREADENTRY32,
};
use windows_sys::Win32::System::JobObjects::IsProcessInJob;
use windows_sys::Win32::System::JobObjects::{
    JobObjectBasicAccountingInformation, QueryInformationJobObject,
    JOBOBJECT_BASIC_ACCOUNTING_INFORMATION,
};
use windows_sys::Win32::System::Threading::{
    OpenThread, ResumeThread, TerminateProcess, WaitForSingleObject, THREAD_SUSPEND_RESUME,
};

pub(in crate::services::owned_process) struct DedicatedJob(super::GlobalJob);

impl DedicatedJob {
    pub(in crate::services::owned_process) fn new() -> Result<Self, OwnedProcessError> {
        super::GlobalJob::new().map(Self)
    }

    pub(in crate::services::owned_process) fn admit(
        &self,
        pid: u32,
    ) -> Result<(), OwnedProcessError> {
        if pid < 2 {
            return Err(OwnedProcessError::Admission);
        }
        let process = ProcessHandle::open(
            pid,
            super::PROCESS_SET_QUOTA
                | super::PROCESS_TERMINATE
                | super::PROCESS_QUERY_LIMITED_INFORMATION,
        )?;
        if unsafe {
            windows_sys::Win32::System::JobObjects::AssignProcessToJobObject(
                self.0.raw(),
                process.0,
            )
        } == 0
        {
            Err(OwnedProcessError::Admission)
        } else {
            Ok(())
        }
    }

    pub(in crate::services::owned_process) fn terminate(&self) -> Result<(), OwnedProcessError> {
        (unsafe { windows_sys::Win32::System::JobObjects::TerminateJobObject(self.0.raw(), 1) }
            != 0)
            .then_some(())
            .ok_or(OwnedProcessError::Admission)
    }

    pub(in crate::services::owned_process) fn is_empty(&self) -> Result<bool, OwnedProcessError> {
        let mut accounting = JOBOBJECT_BASIC_ACCOUNTING_INFORMATION::default();
        let mut returned = 0_u32;
        let ok = unsafe {
            QueryInformationJobObject(
                self.0.raw(),
                JobObjectBasicAccountingInformation,
                (&raw mut accounting).cast(),
                std::mem::size_of_val(&accounting) as u32,
                &raw mut returned,
            )
        };
        if ok == 0 {
            return Err(OwnedProcessError::Admission);
        }
        Ok(accounting.ActiveProcesses == 0)
    }

    #[cfg(test)]
    pub(in crate::services::owned_process) fn contains(&self, pid: u32) -> bool {
        let Ok(process) = ProcessHandle::open(pid, super::PROCESS_QUERY_LIMITED_INFORMATION) else {
            return false;
        };
        let mut contained = 0;
        (unsafe { IsProcessInJob(process.0, self.0.raw(), &mut contained) }) != 0 && contained != 0
    }
}

pub(in crate::services::owned_process) fn resume_suspended_process(
    pid: u32,
) -> Result<(), OwnedProcessError> {
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) };
    if snapshot == INVALID_HANDLE_VALUE {
        return Err(OwnedProcessError::Admission);
    }
    let mut entry = THREADENTRY32 {
        dwSize: std::mem::size_of::<THREADENTRY32>() as u32,
        ..unsafe { std::mem::zeroed() }
    };
    let mut found = false;
    let mut current = unsafe { Thread32First(snapshot, &mut entry) } != 0;
    while current {
        if entry.th32OwnerProcessID == pid {
            let thread = unsafe { OpenThread(THREAD_SUSPEND_RESUME, 0, entry.th32ThreadID) };
            if !thread.is_null() {
                let resumed = unsafe { ResumeThread(thread) } != u32::MAX;
                unsafe { CloseHandle(thread) };
                if resumed {
                    found = true;
                    break;
                }
            }
        }
        current = unsafe { Thread32Next(snapshot, &mut entry) } != 0;
    }
    unsafe { CloseHandle(snapshot) };
    found.then_some(()).ok_or(OwnedProcessError::Admission)
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
    let Ok(process) = ProcessHandle::open(pid, super::PROCESS_QUERY_LIMITED_INFORMATION) else {
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
            let process = ProcessHandle::open(
                *pid,
                super::PROCESS_TERMINATE | super::PROCESS_QUERY_LIMITED_INFORMATION,
            )
            .ok()?;
            let mut contained = 0;
            let in_job = unsafe { IsProcessInJob(process.0, job.raw(), &mut contained) } != 0
                && contained != 0;
            in_job.then_some(process)
        })
        .collect::<Vec<_>>();
    for process in &processes {
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
