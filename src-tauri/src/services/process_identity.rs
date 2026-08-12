use std::path::{Path, PathBuf};
use std::time::Duration;
use sysinfo::{Pid, System};

const CAPTURE_ATTEMPTS: usize = 20;
const CAPTURE_DELAY: Duration = Duration::from_millis(10);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessIdentity {
    pid: u32,
    parent_pid: u32,
    start_time: u64,
    executable: PathBuf,
}

impl ProcessIdentity {
    pub fn capture_child(pid: u32, parent_pid: u32, expected: &Path) -> Option<Self> {
        if pid < 2 || parent_pid < 2 {
            return None;
        }
        let expected = std::fs::canonicalize(expected).ok()?;
        for _ in 0..CAPTURE_ATTEMPTS {
            let mut system = System::new();
            let system_pid = Pid::from_u32(pid);
            system.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[system_pid]), true);
            if let Some(identity) = Self::from_system(&system, pid) {
                return (identity.parent_pid == parent_pid && identity.executable == expected)
                    .then_some(identity);
            }
            std::thread::sleep(CAPTURE_DELAY);
        }
        None
    }

    pub fn from_system(system: &System, pid: u32) -> Option<Self> {
        let process = system.process(Pid::from_u32(pid))?;
        Some(Self {
            pid,
            parent_pid: process.parent()?.as_u32(),
            start_time: process.start_time(),
            executable: std::fs::canonicalize(process.exe()?).ok()?,
        })
    }

    pub fn is_current(&self, system: &System) -> bool {
        Self::from_system(system, self.pid).is_some_and(|current| current == *self)
    }

    pub fn pid(&self) -> u32 {
        self.pid
    }

    #[cfg(test)]
    pub(crate) fn from_parts(
        pid: u32,
        parent_pid: u32,
        start_time: u64,
        executable: PathBuf,
    ) -> Self {
        Self {
            pid,
            parent_pid,
            start_time,
            executable,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_rejects_pid_reuse_and_a_different_executable() {
        let expected = ProcessIdentity::from_parts(42, 7, 100, PathBuf::from("beaver-helper"));

        assert_ne!(
            expected,
            ProcessIdentity::from_parts(42, 7, 101, PathBuf::from("beaver-helper"))
        );
        assert_ne!(
            expected,
            ProcessIdentity::from_parts(42, 7, 100, PathBuf::from("other-program"))
        );
        assert_ne!(
            expected,
            ProcessIdentity::from_parts(42, 8, 100, PathBuf::from("beaver-helper"))
        );
    }
}
