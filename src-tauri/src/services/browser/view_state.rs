#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ViewPhase {
    Cold,
    Creating,
    Ready,
    Closing,
    Closed,
}

pub(super) struct ViewState {
    phase: ViewPhase,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            phase: ViewPhase::Cold,
        }
    }
}

impl ViewState {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    pub(super) fn is_ready(&self) -> bool {
        self.phase == ViewPhase::Ready
    }

    #[cfg(test)]
    pub(super) fn phase(&self) -> ViewPhase {
        self.phase
    }

    pub(super) fn begin_creation(&mut self) -> bool {
        self.transition(ViewPhase::Cold, ViewPhase::Creating)
    }

    pub(super) fn mark_creation_failed(&mut self) -> bool {
        self.transition(ViewPhase::Creating, ViewPhase::Cold)
    }

    pub(super) fn mark_ready(&mut self) -> bool {
        self.transition(ViewPhase::Creating, ViewPhase::Ready)
    }

    pub(super) fn begin_closing(&mut self) -> bool {
        if !matches!(self.phase, ViewPhase::Creating | ViewPhase::Ready) {
            return false;
        }
        self.phase = ViewPhase::Closing;
        true
    }

    pub(super) fn mark_closed(&mut self) -> bool {
        self.transition(ViewPhase::Closing, ViewPhase::Closed)
    }

    fn transition(&mut self, expected: ViewPhase, next: ViewPhase) -> bool {
        if self.phase != expected {
            return false;
        }
        self.phase = next;
        true
    }
}
