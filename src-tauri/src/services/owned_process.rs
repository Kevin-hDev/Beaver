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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OwnedProcessError {
    Spawn(std::io::ErrorKind),
    Admission,
}

pub struct OwnedProcess;

impl OwnedProcess {
    #[cfg(unix)]
    pub(crate) fn adopt_existing(pid: u32) -> Result<(), OwnedProcessError> {
        platform::admit(pid)
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

    #[cfg(windows)]
    pub(crate) fn spawn_conpty<T: windows_spawn::AsPseudoConsole>(
        command: &mut windows_spawn::Command,
        pseudoconsole: &T,
    ) -> Result<windows_spawn::Child, OwnedProcessError> {
        platform::spawn_conpty(command, pseudoconsole)
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
