use crate::contract::{InstallerEvent, InstallerPhase, InstallerSnapshot, ProgressMode};
use std::time::Instant;
use tokio_util::sync::CancellationToken;

const STEP_COUNT: u8 = 5;

pub(super) struct State {
    pub(super) snapshot: InstallerSnapshot,
    pub(super) sequence: u64,
    pub(super) cancel: Option<CancellationToken>,
    pub(super) step_started: Option<Instant>,
    pub(super) completed_step_durations_ms: Vec<u64>,
}

impl State {
    pub(super) fn new(
        version: &str,
        destination: &str,
        installed_version: Option<String>,
        beaver_running: bool,
    ) -> Self {
        Self {
            snapshot: InstallerSnapshot {
                version: version.to_string(),
                destination: destination.to_string(),
                installed_version,
                beaver_running,
                phase: InstallerPhase::Ready,
                step_index: 0,
                step_count: STEP_COUNT,
                progress_mode: ProgressMode::Indeterminate,
                percent: None,
                can_cancel: false,
                outcome: None,
                error_key: None,
            },
            sequence: 0,
            cancel: None,
            step_started: None,
            completed_step_durations_ms: Vec::with_capacity(STEP_COUNT as usize),
        }
    }

    pub(super) fn event(&self, log_key: Option<&'static str>) -> InstallerEvent {
        InstallerEvent {
            sequence: self.sequence,
            snapshot: self.snapshot.clone(),
            completed_step_durations_ms: self.completed_step_durations_ms.clone(),
            log_key: log_key.map(str::to_string),
        }
    }
}

pub(super) fn set_phase(
    state: &mut State,
    phase: InstallerPhase,
    step_index: u8,
    progress_mode: ProgressMode,
    percent: Option<u8>,
    can_cancel: bool,
) {
    if step_index > state.snapshot.step_index {
        complete_step(state);
        state.step_started = Some(Instant::now());
    }
    state.snapshot.phase = phase;
    state.snapshot.step_index = step_index;
    state.snapshot.progress_mode = progress_mode;
    state.snapshot.percent = percent;
    state.snapshot.can_cancel = can_cancel;
    state.sequence = state.sequence.saturating_add(1);
}

pub(super) fn complete_step(state: &mut State) {
    if let Some(started) = state.step_started.take() {
        state
            .completed_step_durations_ms
            .push(started.elapsed().as_millis().min(u64::MAX as u128) as u64);
    }
}

pub(super) fn allowed_transition(from: InstallerPhase, to: InstallerPhase) -> bool {
    matches!(
        (from, to),
        (InstallerPhase::Checking, InstallerPhase::Downloading)
            | (InstallerPhase::Downloading, InstallerPhase::Downloading)
            | (InstallerPhase::Downloading, InstallerPhase::Verifying)
            | (InstallerPhase::Verifying, InstallerPhase::Installing)
            | (InstallerPhase::Installing, InstallerPhase::Installing)
            | (InstallerPhase::Installing, InstallerPhase::Finishing)
            | (InstallerPhase::Finishing, InstallerPhase::Completed)
            | (InstallerPhase::Cancelling, InstallerPhase::Cancelled)
            | (InstallerPhase::Checking, InstallerPhase::Failed)
            | (InstallerPhase::Downloading, InstallerPhase::Failed)
            | (InstallerPhase::Verifying, InstallerPhase::Failed)
            | (InstallerPhase::Installing, InstallerPhase::Failed)
            | (InstallerPhase::Finishing, InstallerPhase::Failed)
    )
}
