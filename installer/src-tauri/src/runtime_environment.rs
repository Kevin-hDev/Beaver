use super::InstallerRuntime;
use crate::contract::InstallerSnapshot;
use crate::error::InstallerError;

impl InstallerRuntime {
    pub fn operation_active(&self) -> bool {
        self.state
            .lock()
            .map(|state| state.cancel.is_some())
            .unwrap_or(true)
    }

    pub fn update_environment(
        &self,
        destination: &str,
        installed_version: Option<String>,
        beaver_running: bool,
    ) -> Result<InstallerSnapshot, InstallerError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| InstallerError::InstallFailed)?;
        if state.cancel.is_some() {
            return Err(InstallerError::InstallFailed);
        }
        state.snapshot.destination = destination.to_string();
        state.snapshot.installed_version = installed_version;
        state.snapshot.beaver_running = beaver_running;
        state.sequence = state.sequence.saturating_add(1);
        Ok(state.snapshot.clone())
    }

    pub fn refresh_running(&self, running: bool) -> Result<InstallerSnapshot, InstallerError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| InstallerError::InstallFailed)?;
        state.snapshot.beaver_running = running;
        Ok(state.snapshot.clone())
    }

    pub(super) fn mark_installed(&self) -> Result<(), InstallerError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| InstallerError::InstallFailed)?;
        state.snapshot.installed_version = Some(state.snapshot.version.clone());
        Ok(())
    }
}
