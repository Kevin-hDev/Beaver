use crate::error::InstallerError;
use std::path::Path;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

const MAX_PROCESSES: usize = 4_096;

pub fn executable_is_running(executable: &Path) -> Result<bool, InstallerError> {
    let expected = executable
        .canonicalize()
        .map_err(|_| InstallerError::InstallFailed)?;
    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_exe(UpdateKind::Always),
    );
    if system.processes().len() > MAX_PROCESSES {
        return Err(InstallerError::InstallFailed);
    }
    Ok(system.processes().values().any(|process| {
        process
            .exe()
            .and_then(|path| path.canonicalize().ok())
            .is_some_and(|path| path == expected)
    }))
}
