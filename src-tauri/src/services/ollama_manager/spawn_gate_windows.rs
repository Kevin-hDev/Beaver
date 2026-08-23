use super::ollama_tree_job::OllamaTreeJob;
use super::process::OllamaProcessError;
use super::spawn_profile::OllamaSpawnAttempt;
#[path = "spawn_gate_windows_support.rs"]
mod support;
use crate::services::owned_process::{OwnedProcess, OwnedProcessIdentity};
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::ptr;
use std::time::Instant;
use windows_sys::Win32::Foundation::{WAIT_OBJECT_0, WAIT_TIMEOUT};
use windows_sys::Win32::System::Threading::{
    CreateProcessW, ResumeThread, WaitForSingleObject, CREATE_NO_WINDOW, CREATE_SUSPENDED,
    CREATE_UNICODE_ENVIRONMENT, PROCESS_INFORMATION, STARTUPINFOW,
};

#[cfg(test)]
pub(super) use support::append_entry;
pub(super) use support::environment_block;
use support::{quote_path, wide_path, wide_string};

pub(crate) struct NativeGatedProcess {
    process: OwnedHandle,
    thread: OwnedHandle,
    tree_job: OllamaTreeJob,
    identity: OwnedProcessIdentity,
    opened: bool,
    termination_requested: bool,
    reaped: bool,
}

pub(crate) fn create(
    attempt: &OllamaSpawnAttempt<'_>,
) -> Result<NativeGatedProcess, OllamaProcessError> {
    create_with_hooks(attempt, || {}, || {})
}

#[cfg(test)]
pub(crate) fn create_with_hooks_for_test(
    attempt: &OllamaSpawnAttempt<'_>,
    before_create: impl FnOnce(),
    after_create: impl FnOnce(),
) -> Result<NativeGatedProcess, OllamaProcessError> {
    create_with_hooks(attempt, before_create, after_create)
}

fn create_with_hooks(
    attempt: &OllamaSpawnAttempt<'_>,
    before_create: impl FnOnce(),
    after_create: impl FnOnce(),
) -> Result<NativeGatedProcess, OllamaProcessError> {
    let executable = wide_path(attempt.profile().executable().path())?;
    let cwd = wide_path(attempt.profile().working_directory().path())?;
    let mut command_line = quote_path(attempt.profile().executable().path());
    command_line.push_str(" serve");
    let mut command_line = wide_string(&command_line)?;
    let environment = environment_block(attempt)?;
    let startup = STARTUPINFOW {
        cb: std::mem::size_of::<STARTUPINFOW>() as u32,
        ..unsafe { std::mem::zeroed() }
    };
    let mut info = unsafe { std::mem::zeroed::<PROCESS_INFORMATION>() };
    let tree_job = OllamaTreeJob::create()?;
    let flags = CREATE_SUSPENDED | CREATE_NO_WINDOW | CREATE_UNICODE_ENVIRONMENT;
    let expected_executable = match attempt.profile().executable().execution_identity() {
        Some(identity) if identity != 0 => identity,
        _ => return Err(OllamaProcessError::Identity),
    };
    before_create();
    let created = unsafe {
        CreateProcessW(
            executable.as_ptr(),
            command_line.as_mut_ptr(),
            ptr::null_mut(),
            ptr::null_mut(),
            0,
            flags,
            environment.as_ptr().cast(),
            cwd.as_ptr(),
            &startup,
            &mut info,
        )
    };
    if created == 0 {
        return Err(OllamaProcessError::Spawn);
    }
    after_create();
    if OwnedProcess::admit_suspended_handle(info.hProcess).is_err() {
        support::terminate_created_process(info.hProcess, info.hThread);
        return Err(OllamaProcessError::Admission);
    }
    if tree_job.assign_process(info.hProcess).is_err() {
        support::terminate_created_process(info.hProcess, info.hThread);
        crate::services::owned_process::release(info.dwProcessId);
        return Err(OllamaProcessError::Admission);
    }
    let identity = match OwnedProcess::identity_from_handle_with_executable(
        info.hProcess,
        expected_executable,
    ) {
        Ok(identity) => identity,
        Err(_) => {
            support::terminate_created_process(info.hProcess, info.hThread);
            return Err(OllamaProcessError::Identity);
        }
    };
    if expected_executable == 0 || identity.executable != expected_executable {
        support::terminate_created_process(info.hProcess, info.hThread);
        crate::services::owned_process::release(info.dwProcessId);
        return Err(OllamaProcessError::Identity);
    }
    Ok(NativeGatedProcess {
        process: unsafe { OwnedHandle::from_raw_handle(info.hProcess) },
        thread: unsafe { OwnedHandle::from_raw_handle(info.hThread) },
        tree_job,
        identity,
        opened: false,
        termination_requested: false,
        reaped: false,
    })
}

impl NativeGatedProcess {
    pub(crate) fn identity(&self) -> OwnedProcessIdentity {
        self.identity
    }

    pub(crate) fn open_gate(&mut self) -> Result<(), OllamaProcessError> {
        if self.opened {
            return Err(OllamaProcessError::Gate);
        }
        let previous = unsafe { ResumeThread(self.thread.as_raw_handle()) };
        if previous == u32::MAX {
            return Err(OllamaProcessError::Gate);
        }
        self.opened = true;
        Ok(())
    }

    pub(crate) fn revalidate(&self, executable: u128) -> Result<(), OllamaProcessError> {
        if executable == 0 || self.identity.executable != executable {
            return Err(OllamaProcessError::Identity);
        }
        let current = OwnedProcess::identity_from_handle_with_executable(
            self.process.as_raw_handle(),
            self.identity.executable,
        )
        .map_err(|_| OllamaProcessError::Identity)?;
        (current == self.identity)
            .then_some(())
            .ok_or(OllamaProcessError::Identity)
    }

    pub(crate) fn wait_for_executable(
        &mut self,
        executable: u128,
        _deadline: Instant,
    ) -> Result<(), OllamaProcessError> {
        self.revalidate(executable)
    }

    pub(crate) fn terminate(&mut self) -> Result<(), OllamaProcessError> {
        if self.reaped || self.termination_requested {
            return Ok(());
        }
        if self.tree_job.terminate().is_ok() {
            self.termination_requested = true;
            return Ok(());
        }
        match unsafe { WaitForSingleObject(self.process.as_raw_handle(), 0) } {
            WAIT_OBJECT_0 => {
                self.termination_requested = true;
                Ok(())
            }
            _ => Err(OllamaProcessError::Gate),
        }
    }

    pub(crate) fn reap(&mut self, deadline: Instant) -> Result<(), OllamaProcessError> {
        if self.reaped {
            return Ok(());
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        let millis = remaining.as_millis().min(u128::from(u32::MAX)) as u32;
        let result = unsafe { WaitForSingleObject(self.process.as_raw_handle(), millis) };
        match result {
            WAIT_OBJECT_0 => {
                self.tree_job.wait(deadline)?;
                crate::services::owned_process::release(self.identity.pid);
                self.reaped = true;
                Ok(())
            }
            WAIT_TIMEOUT => Err(OllamaProcessError::Reap),
            _ => Err(OllamaProcessError::Reap),
        }
    }

    pub(crate) fn terminate_and_reap(
        &mut self,
        deadline: Instant,
    ) -> Result<(), OllamaProcessError> {
        self.terminate()?;
        self.reap(deadline)
    }
}
