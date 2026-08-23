use super::process::OllamaProcessError;
use std::os::windows::io::{AsHandle, AsRawHandle};
use std::time::Instant;
use windows_sys::Win32::Foundation::{HANDLE, WAIT_OBJECT_0};
use windows_sys::Win32::System::JobObjects::AssignProcessToJobObject;
use windows_sys::Win32::System::Threading::WaitForSingleObject;

pub(crate) struct OllamaTreeJob {
    job: windows_spawn::Job,
}

impl OllamaTreeJob {
    pub(crate) fn create() -> Result<Self, OllamaProcessError> {
        let job = windows_spawn::Job::create().map_err(|_| OllamaProcessError::Admission)?;
        // This private lifetime is the restart boundary: closing it must never
        // leave a model runner inside Beaver's longer-lived global Job.
        job.set_kill_on_close(true)
            .map_err(|_| OllamaProcessError::Admission)?;
        Ok(Self { job })
    }

    pub(crate) fn assign_process(&self, process: HANDLE) -> Result<(), OllamaProcessError> {
        let assigned = unsafe { AssignProcessToJobObject(self.raw(), process) };
        (assigned != 0)
            .then_some(())
            .ok_or(OllamaProcessError::Admission)
    }

    pub(crate) fn terminate(&self) -> Result<(), OllamaProcessError> {
        self.job.terminate(1).map_err(|_| OllamaProcessError::Gate)
    }

    pub(crate) fn wait(&self, deadline: Instant) -> Result<(), OllamaProcessError> {
        let remaining = deadline.saturating_duration_since(Instant::now());
        let millis = remaining.as_millis().min(u128::from(u32::MAX)) as u32;
        (unsafe { WaitForSingleObject(self.raw(), millis) } == WAIT_OBJECT_0)
            .then_some(())
            .ok_or(OllamaProcessError::Reap)
    }

    #[cfg(test)]
    pub(crate) fn terminate_and_wait(&self, deadline: Instant) -> Result<(), OllamaProcessError> {
        self.terminate()?;
        self.wait(deadline)
    }

    fn raw(&self) -> HANDLE {
        self.job.as_handle().as_raw_handle()
    }
}
