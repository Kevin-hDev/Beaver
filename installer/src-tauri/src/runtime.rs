use crate::contract::{InstallerEvent, InstallerOutcome, InstallerPhase, ProgressMode};
use crate::error::InstallerError;
use std::sync::Mutex;
use tokio_util::sync::CancellationToken;

#[path = "runtime_state.rs"]
mod state;
use state::{allowed_transition, complete_step, set_phase, State};
#[path = "runtime_environment.rs"]
mod environment;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PlatformKind {
    Macos,
    Windows,
}

pub struct InstallerRuntime {
    platform: PlatformKind,
    state: Mutex<State>,
}

impl InstallerRuntime {
    pub fn new(
        version: &str,
        destination: &str,
        installed_version: Option<String>,
        beaver_running: bool,
        platform: PlatformKind,
    ) -> Self {
        Self {
            platform,
            state: Mutex::new(State::new(
                version,
                destination,
                installed_version,
                beaver_running,
            )),
        }
    }

    pub fn snapshot(&self) -> InstallerEvent {
        self.state.lock().expect("installer state").event(None)
    }

    pub fn operation_active(&self) -> bool {
        self.state
            .lock()
            .map(|state| state.cancel.is_some())
            .unwrap_or(true)
    }

    pub fn begin(&self) -> Result<CancellationToken, InstallerError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| InstallerError::InstallFailed)?;
        if state.cancel.is_some()
            || state.snapshot.beaver_running
            || !matches!(
                state.snapshot.phase,
                InstallerPhase::Ready | InstallerPhase::Failed | InstallerPhase::Cancelled
            )
        {
            return Err(InstallerError::InstallFailed);
        }
        let cancel = CancellationToken::new();
        state.cancel = Some(cancel.clone());
        state.completed_step_durations_ms.clear();
        state.snapshot.outcome = None;
        state.snapshot.error_key = None;
        set_phase(
            &mut state,
            InstallerPhase::Checking,
            1,
            ProgressMode::Indeterminate,
            None,
            true,
        );
        Ok(cancel)
    }

    pub fn downloading(&self, percent: u8) -> Result<InstallerEvent, InstallerError> {
        self.publish(
            InstallerPhase::Downloading,
            2,
            ProgressMode::Determinate,
            Some(percent.min(100)),
            true,
            Some("installer.log.downloading"),
        )
    }

    pub fn verifying(&self) -> Result<InstallerEvent, InstallerError> {
        self.publish(
            InstallerPhase::Verifying,
            3,
            ProgressMode::Indeterminate,
            None,
            true,
            Some("installer.log.verifying"),
        )
    }

    pub fn installing(&self, staging_cancellable: bool) -> Result<InstallerEvent, InstallerError> {
        self.publish(
            InstallerPhase::Installing,
            4,
            ProgressMode::Indeterminate,
            None,
            self.platform == PlatformKind::Macos && staging_cancellable,
            Some("installer.log.installing"),
        )
    }

    pub fn begin_swap(&self) -> Result<InstallerEvent, InstallerError> {
        self.publish(
            InstallerPhase::Installing,
            4,
            ProgressMode::Indeterminate,
            None,
            false,
            None,
        )
    }

    pub fn finishing(&self) -> Result<InstallerEvent, InstallerError> {
        self.publish(
            InstallerPhase::Finishing,
            5,
            ProgressMode::Indeterminate,
            None,
            false,
            Some("installer.log.finishing"),
        )
    }

    pub fn cancel(&self) -> bool {
        let Ok(mut state) = self.state.lock() else {
            return false;
        };
        if !state.snapshot.can_cancel {
            return false;
        }
        let Some(cancel) = state.cancel.clone() else {
            return false;
        };
        cancel.cancel();
        state.snapshot.phase = InstallerPhase::Cancelling;
        state.snapshot.can_cancel = false;
        state.sequence = state.sequence.saturating_add(1);
        true
    }

    pub fn cancelled(
        &self,
        _operation: CancellationToken,
    ) -> Result<InstallerEvent, InstallerError> {
        self.finish(InstallerPhase::Cancelled, None, None)
    }

    pub fn fail(
        &self,
        _operation: CancellationToken,
        error_key: &'static str,
    ) -> Result<InstallerEvent, InstallerError> {
        self.finish(InstallerPhase::Failed, None, Some(error_key))
    }

    pub fn succeed(
        &self,
        _operation: CancellationToken,
        outcome: InstallerOutcome,
    ) -> Result<InstallerEvent, InstallerError> {
        self.mark_installed()?;
        self.finish(InstallerPhase::Completed, Some(outcome), None)
    }

    fn publish(
        &self,
        phase: InstallerPhase,
        step_index: u8,
        progress_mode: ProgressMode,
        percent: Option<u8>,
        can_cancel: bool,
        log_key: Option<&'static str>,
    ) -> Result<InstallerEvent, InstallerError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| InstallerError::InstallFailed)?;
        if state.cancel.is_none() || !allowed_transition(state.snapshot.phase, phase) {
            return Err(InstallerError::InstallFailed);
        }
        let phase_changed = state.snapshot.phase != phase;
        set_phase(
            &mut state,
            phase,
            step_index,
            progress_mode,
            percent,
            can_cancel,
        );
        Ok(state.event(log_key.filter(|_| phase_changed)))
    }

    fn finish(
        &self,
        phase: InstallerPhase,
        outcome: Option<InstallerOutcome>,
        error_key: Option<&'static str>,
    ) -> Result<InstallerEvent, InstallerError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| InstallerError::InstallFailed)?;
        if state.cancel.take().is_none() || !allowed_transition(state.snapshot.phase, phase) {
            return Err(InstallerError::InstallFailed);
        }
        if phase == InstallerPhase::Completed {
            complete_step(&mut state);
        }
        state.snapshot.phase = phase;
        state.snapshot.can_cancel = false;
        state.snapshot.percent = (phase == InstallerPhase::Completed).then_some(100);
        state.snapshot.outcome = outcome;
        state.snapshot.error_key = error_key.map(str::to_string);
        state.sequence = state.sequence.saturating_add(1);
        Ok(state.event(None))
    }
}
