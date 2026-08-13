use super::super::CefUnavailableCategory;
use super::process_state::{
    MacProcessActions, MacProcessObservation, MacSignalObservation, MacSystemProcessActions,
};
use crate::services::browser::native_paths::MacHelperExecutables;
#[cfg(test)]
use std::path::Path;
use std::path::PathBuf;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::services::browser) struct MacProcessIdentity {
    pub(super) pid: u32,
    pub(super) parent_pid: u32,
    pub(super) started_at: u64,
    pub(super) process_group: u32,
    pub(super) executable: PathBuf,
}

impl MacProcessIdentity {
    pub(in crate::services::browser) fn read(pid: u32) -> Result<Self, CefUnavailableCategory> {
        super::process_state::read_active_identity(pid)
    }

    pub(in crate::services::browser) fn validate(
        pid: u32,
        parent_pid: u32,
        started_at: u64,
        process_group: u32,
        executables: &MacHelperExecutables,
    ) -> Result<Self, CefUnavailableCategory> {
        let identity = Self::read(pid)?;
        if parent_pid == 0
            || started_at == 0
            || process_group != pid
            || identity.parent_pid != parent_pid
            || identity.started_at != started_at
            || identity.process_group != process_group
            || !executables.contains(&identity.executable)
        {
            Err(CefUnavailableCategory::Admission)
        } else {
            Ok(identity)
        }
    }

    pub(super) fn revalidate(&self) -> Result<(), CefUnavailableCategory> {
        match MacSystemProcessActions.revalidate_before_signal(self) {
            MacSignalObservation::Ready => Ok(()),
            MacSignalObservation::Stopped | MacSignalObservation::Unknown => {
                Err(CefUnavailableCategory::Reaper)
            }
        }
    }

    pub(super) fn kill_group(&self) -> Result<(), CefUnavailableCategory> {
        self.revalidate()?;
        MacSystemProcessActions.signal_group(self).map(|_| ())
    }

    pub(super) fn is_alive(&self) -> Result<bool, CefUnavailableCategory> {
        match MacSystemProcessActions.observe(self) {
            MacProcessObservation::Alive => Ok(true),
            MacProcessObservation::Stopped => Ok(false),
            MacProcessObservation::Unknown => Err(CefUnavailableCategory::Reaper),
        }
    }

    #[cfg(test)]
    pub(in crate::services::browser) fn test_pid(&self) -> u32 {
        self.pid
    }

    #[cfg(test)]
    pub(in crate::services::browser) fn test_parent_pid(&self) -> u32 {
        self.parent_pid
    }

    #[cfg(test)]
    pub(in crate::services::browser) fn test_started_at(&self) -> u64 {
        self.started_at
    }

    #[cfg(test)]
    pub(in crate::services::browser) fn test_process_group(&self) -> u32 {
        self.process_group
    }

    #[cfg(test)]
    pub(in crate::services::browser) fn test_executable(&self) -> &Path {
        &self.executable
    }

    #[cfg(test)]
    pub(in crate::services::browser) fn test_is_alive(
        &self,
    ) -> Result<bool, CefUnavailableCategory> {
        self.is_alive()
    }
}
