#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) enum RuntimePhase {
    #[default]
    Cold,
    ApplicationPrepared,
    Running,
    Stopping,
    Stopped,
    Failed,
}

#[derive(Debug, Default)]
pub(super) struct Lifecycle {
    phase: RuntimePhase,
}

impl Lifecycle {
    pub(super) fn phase(&self) -> RuntimePhase {
        self.phase
    }

    pub(super) fn mark_application_prepared(&mut self) -> bool {
        if self.phase != RuntimePhase::Cold {
            return false;
        }
        self.phase = RuntimePhase::ApplicationPrepared;
        true
    }

    pub(super) fn mark_running(&mut self) -> bool {
        if self.phase != RuntimePhase::ApplicationPrepared {
            self.phase = RuntimePhase::Failed;
            return false;
        }
        self.phase = RuntimePhase::Running;
        true
    }

    pub(super) fn mark_failed(&mut self) -> bool {
        if !matches!(
            self.phase,
            RuntimePhase::ApplicationPrepared | RuntimePhase::Running
        ) {
            return false;
        }
        self.phase = RuntimePhase::Failed;
        true
    }

    pub(super) fn begin_stopping(&mut self) -> bool {
        if !matches!(
            self.phase,
            RuntimePhase::ApplicationPrepared | RuntimePhase::Running | RuntimePhase::Failed
        ) {
            return false;
        }
        self.phase = RuntimePhase::Stopping;
        true
    }

    pub(super) fn mark_stopped(&mut self) -> bool {
        if self.phase != RuntimePhase::Stopping {
            return false;
        }
        self.phase = RuntimePhase::Stopped;
        true
    }
}
