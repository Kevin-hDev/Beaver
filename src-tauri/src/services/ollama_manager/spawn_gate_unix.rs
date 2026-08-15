use super::process::OllamaProcessError;
use super::spawn_profile::OllamaSpawnAttempt;
#[path = "spawn_gate_unix_support.rs"]
mod support;
#[cfg(test)]
#[path = "spawn_gate_unix_support_tests.rs"]
mod support_tests;
use crate::services::owned_process::{OwnedProcess, OwnedProcessError, OwnedProcessIdentity};
use std::fs::File;
use std::io::Write;
use std::os::fd::{AsRawFd, FromRawFd};
use std::time::Instant;

use support::{
    c_string, child_exec, close, environment_block, pipe, stable_executable_link, wait_blocking,
    wait_nonblocking, StableExecutableLink,
};

pub(crate) struct NativeGatedProcess {
    pid: libc::pid_t,
    gate: Option<File>,
    #[cfg(test)]
    test_gate_keepalive: Option<File>,
    #[cfg(test)]
    force_reap_failure: bool,
    exec_link: Option<StableExecutableLink>,
    identity: OwnedProcessIdentity,
    opened: bool,
    reaped: bool,
}

pub(crate) fn create(
    attempt: &OllamaSpawnAttempt<'_>,
) -> Result<NativeGatedProcess, OllamaProcessError> {
    create_with_admitter(attempt, OwnedProcess::adopt_existing)
}

#[cfg(test)]
pub(crate) fn create_with_admitter_for_test(
    attempt: &OllamaSpawnAttempt<'_>,
    admit: impl Fn(u32) -> Result<(), OwnedProcessError>,
) -> Result<NativeGatedProcess, OllamaProcessError> {
    create_with_admitter(attempt, admit)
}

fn create_with_admitter(
    attempt: &OllamaSpawnAttempt<'_>,
    admit: impl Fn(u32) -> Result<(), OwnedProcessError>,
) -> Result<NativeGatedProcess, OllamaProcessError> {
    if attempt.profile().executable().stable_handle().is_none() {
        return Err(OllamaProcessError::Identity);
    }
    if !attempt.profile().executable().stable_path_is_current() {
        return Err(OllamaProcessError::Identity);
    }
    let executable_identity = attempt.profile().executable().identity().value();
    let exec_link =
        stable_executable_link(attempt.profile().executable().path(), executable_identity)
            .map_err(|_| OllamaProcessError::Identity)?;
    let linked_executable = c_string(exec_link.path())?;
    let stdio = std::fs::OpenOptions::new()
        .write(true)
        .open("/dev/null")
        .map_err(|_| OllamaProcessError::Spawn)?;
    let stdio_fd = stdio.as_raw_fd();
    let cwd = c_string(attempt.profile().working_directory().path())?;
    let serve = c_string("serve")?;
    let args = [linked_executable.as_ptr(), serve.as_ptr(), std::ptr::null()];
    let environment = environment_block(attempt)?;
    let environment_ptrs = environment
        .iter()
        .map(|value| value.as_ptr())
        .chain(std::iter::once(std::ptr::null()))
        .collect::<Vec<_>>();
    let (read_fd, write_fd) = pipe().map_err(|_| OllamaProcessError::Gate)?;
    let pid = unsafe { libc::fork() };
    if pid < 0 {
        close(read_fd);
        close(write_fd);
        return Err(OllamaProcessError::Spawn);
    }
    if pid == 0 {
        child_exec(
            read_fd,
            write_fd,
            cwd.as_ptr(),
            linked_executable.as_ptr(),
            args.as_ptr(),
            environment_ptrs.as_ptr(),
            stdio_fd,
        );
    }
    close(read_fd);
    if unsafe { libc::setpgid(pid, pid) } != 0 {
        let mut process = support::failed_process(pid, write_fd);
        let _ = process
            .terminate_and_reap(Instant::now() + super::constants::PROCESS_REAP_FALLBACK_TIMEOUT);
        return Err(OllamaProcessError::Admission);
    }
    if admit(pid as u32).is_err() {
        let mut process = support::failed_process(pid, write_fd);
        let _ = process
            .terminate_and_reap(Instant::now() + super::constants::PROCESS_REAP_FALLBACK_TIMEOUT);
        return Err(OllamaProcessError::Admission);
    }
    let identity = match OwnedProcess::identity_with_executable(pid as u32, executable_identity) {
        Ok(identity) => identity,
        Err(_) => {
            let mut process = support::failed_process(pid, write_fd);
            let _ = process.terminate_and_reap(
                Instant::now() + super::constants::PROCESS_REAP_FALLBACK_TIMEOUT,
            );
            return Err(OllamaProcessError::Identity);
        }
    };
    Ok(NativeGatedProcess {
        pid,
        gate: Some(unsafe { File::from_raw_fd(write_fd) }),
        #[cfg(test)]
        test_gate_keepalive: None,
        #[cfg(test)]
        force_reap_failure: false,
        exec_link: Some(exec_link),
        identity,
        opened: false,
        reaped: false,
    })
}

impl NativeGatedProcess {
    pub(crate) fn identity(&self) -> OwnedProcessIdentity {
        self.identity
    }

    pub(crate) fn open_gate(&mut self) -> Result<(), OllamaProcessError> {
        let mut gate = self.gate.take().ok_or(OllamaProcessError::Gate)?;
        gate.write_all(&[1]).map_err(|_| OllamaProcessError::Gate)?;
        self.opened = true;
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn close_gate_for_test(&mut self) {
        self.test_gate_keepalive = self.gate.take();
        self.gate = std::fs::File::open("/dev/null").ok();
    }

    pub(crate) fn revalidate(&self, executable: u128) -> Result<(), OllamaProcessError> {
        let current = if self.opened {
            OwnedProcess::identity(self.identity.pid)
        } else {
            OwnedProcess::identity_with_executable(self.identity.pid, executable)
        }
        .map_err(|_| OllamaProcessError::Identity)?;
        if current != self.identity || current.executable != executable {
            return Err(OllamaProcessError::Identity);
        }
        Ok(())
    }

    pub(crate) fn wait_for_executable(
        &self,
        executable: u128,
        deadline: Instant,
    ) -> Result<(), OllamaProcessError> {
        while Instant::now() < deadline {
            let current = OwnedProcess::identity(self.identity.pid)
                .map_err(|_| OllamaProcessError::Identity)?;
            if current.pid != self.identity.pid
                || current.native_scope != self.identity.native_scope
                || current.native_start_time != self.identity.native_start_time
            {
                return Err(OllamaProcessError::Identity);
            }
            if current.executable == executable {
                return Ok(());
            }
            std::thread::yield_now();
        }
        Err(OllamaProcessError::Gate)
    }

    pub(crate) fn terminate_and_reap(
        &mut self,
        deadline: Instant,
    ) -> Result<(), OllamaProcessError> {
        #[cfg(test)]
        if self.force_reap_failure {
            return Err(OllamaProcessError::Reap);
        }
        if self.reaped {
            return Ok(());
        }
        self.gate.take();
        unsafe {
            libc::kill(-self.pid, libc::SIGTERM);
            libc::kill(self.pid, libc::SIGTERM);
        }
        while Instant::now() < deadline {
            match wait_nonblocking(self.pid)? {
                Some(_) => {
                    crate::services::owned_process::release(self.identity.pid);
                    self.reaped = true;
                    return Ok(());
                }
                None => std::thread::yield_now(),
            }
        }
        unsafe {
            libc::kill(-self.pid, libc::SIGKILL);
            libc::kill(self.pid, libc::SIGKILL);
        }
        wait_blocking(self.pid)?;
        crate::services::owned_process::release(self.identity.pid);
        self.reaped = true;
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn force_reap_failure_for_test(&mut self) {
        self.force_reap_failure = true;
    }
}
