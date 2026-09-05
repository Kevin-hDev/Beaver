use super::process_tree::{self, ProcessKind};
use std::process::{Child, Command};

#[cfg(target_os = "macos")]
#[path = "owned_process_macos.rs"]
mod macos;
#[cfg(unix)]
#[path = "owned_process_unix.rs"]
mod platform;
#[cfg(windows)]
#[path = "owned_process_windows.rs"]
mod platform;
#[cfg(all(test, unix))]
#[path = "owned_process_unix_recovery_tests.rs"]
mod unix_recovery_tests;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OwnedProcessError {
    Spawn(std::io::ErrorKind),
    Admission,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) struct OwnedProcessIdentity {
    pub(crate) pid: u32,
    pub(crate) native_scope: u64,
    pub(crate) native_start_time: u64,
    pub(crate) executable: u128,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OwnedProcessInspection {
    Owned(OwnedProcessIdentity),
    Unowned,
}

pub struct OwnedProcess;

#[path = "owned_process_scope.rs"]
mod scope;
pub(crate) use scope::OwnedProcessScope;

impl OwnedProcess {
    #[cfg(unix)]
    pub(crate) fn adopt_existing(pid: u32) -> Result<(), OwnedProcessError> {
        platform::admit(pid)
    }

    pub(crate) fn identity(pid: u32) -> Result<OwnedProcessIdentity, OwnedProcessError> {
        platform::identity(pid)
    }

    pub(crate) fn inspect_for_recovery(
        pid: u32,
        expected_start_time: u64,
    ) -> Result<OwnedProcessInspection, OwnedProcessError> {
        #[cfg(unix)]
        return platform::inspect_for_recovery(pid, expected_start_time);
        #[cfg(windows)]
        {
            let _ = expected_start_time;
            platform::inspect_for_recovery(pid)
        }
    }

    #[cfg(unix)]
    pub(crate) fn identity_with_executable(
        pid: u32,
        executable: u128,
    ) -> Result<OwnedProcessIdentity, OwnedProcessError> {
        platform::identity_with_executable(pid, executable)
    }

    pub(crate) fn recover_exact(
        expected: OwnedProcessIdentity,
        deadline: std::time::Instant,
    ) -> Result<(), OwnedProcessError> {
        Self::recover_exact_with_cancel(expected, deadline, || false)
    }

    pub(crate) fn recover_exact_with_cancel(
        expected: OwnedProcessIdentity,
        deadline: std::time::Instant,
        cancelled: impl Fn() -> bool,
    ) -> Result<(), OwnedProcessError> {
        platform::recover_exact_with_cancel(expected, deadline, &cancelled)
    }

    pub(crate) fn reap_exited_child(pid: u32) -> Result<bool, OwnedProcessError> {
        platform::reap_exited_child(pid)
    }

    pub(crate) fn signal_exact(
        expected: OwnedProcessIdentity,
        force: bool,
    ) -> Result<(), OwnedProcessError> {
        platform::signal_exact(expected, force)
    }

    pub(crate) fn process_exists(pid: u32) -> bool {
        platform::process_exists(pid)
    }

    pub fn spawn(command: &mut Command, kind: ProcessKind) -> Result<Child, OwnedProcessError> {
        Self::spawn_with_admitter(command, kind, platform::admit)
    }

    pub async fn spawn_tokio(
        command: &mut tokio::process::Command,
        kind: ProcessKind,
    ) -> Result<tokio::process::Child, OwnedProcessError> {
        process_tree::configure_tokio(command);
        let mut child = command
            .spawn()
            .map_err(|error| OwnedProcessError::Spawn(error.kind()))?;
        let Some(pid) = child.id() else {
            process_tree::terminate_tokio(&mut child, kind).await;
            return Err(OwnedProcessError::Admission);
        };
        if platform::admit(pid).is_err() {
            process_tree::terminate_tokio(&mut child, kind).await;
            return Err(OwnedProcessError::Admission);
        }
        Ok(child)
    }

    pub async fn spawn_tokio_scoped(
        command: &mut tokio::process::Command,
        kind: ProcessKind,
    ) -> Result<(tokio::process::Child, OwnedProcessScope), OwnedProcessError> {
        OwnedProcessScope::spawn_tokio(command, kind).await
    }

    #[cfg(windows)]
    pub(crate) fn spawn_conpty<T: windows_spawn::AsPseudoConsole>(
        command: &mut windows_spawn::Command,
        pseudoconsole: &T,
    ) -> Result<windows_spawn::Child, OwnedProcessError> {
        platform::spawn_conpty(command, pseudoconsole)
    }

    #[cfg(windows)]
    pub(crate) fn admit_suspended_handle(
        process: windows_sys::Win32::Foundation::HANDLE,
    ) -> Result<(), OwnedProcessError> {
        platform::admit_suspended_handle(process)
    }

    #[cfg(windows)]
    pub(crate) fn identity_from_handle_with_executable(
        process: windows_sys::Win32::Foundation::HANDLE,
        executable: u128,
    ) -> Result<OwnedProcessIdentity, OwnedProcessError> {
        platform::identity_from_handle_with_executable(process, executable)
    }

    fn spawn_with_admitter(
        command: &mut Command,
        kind: ProcessKind,
        admit: fn(u32) -> Result<(), OwnedProcessError>,
    ) -> Result<Child, OwnedProcessError> {
        process_tree::configure(command);
        let mut child = command
            .spawn()
            .map_err(|error| OwnedProcessError::Spawn(error.kind()))?;
        if admit(child.id()).is_err() {
            process_tree::terminate(&mut child, kind);
            return Err(OwnedProcessError::Admission);
        }
        Ok(child)
    }

    #[cfg(test)]
    pub(crate) fn spawn_with_admitter_for_test(
        command: &mut Command,
        kind: ProcessKind,
        admit: fn(u32) -> Result<(), OwnedProcessError>,
    ) -> Result<Child, OwnedProcessError> {
        Self::spawn_with_admitter(command, kind, admit)
    }

    #[cfg(test)]
    pub(crate) fn is_confined_for_test(pid: u32) -> bool {
        platform::is_confined(pid)
    }
}

pub(crate) fn release(pid: u32) {
    platform::release(pid);
}

#[cfg(windows)]
pub(crate) fn is_confined(pid: u32) -> bool {
    platform::is_confined(pid)
}

#[cfg(windows)]
pub(crate) fn terminate_confined(pids: &[u32], deadline: std::time::Instant) -> usize {
    platform::terminate_confined(pids, deadline)
}

#[cfg(target_os = "macos")]
pub(crate) fn signal_is_safe(pid: u32) -> bool {
    macos::signal_is_safe(pid)
}

#[cfg(all(unix, not(target_os = "macos")))]
pub(crate) fn signal_is_safe(_pid: u32) -> bool {
    true
}
