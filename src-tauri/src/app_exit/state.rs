use std::sync::atomic::{AtomicU8, Ordering};

const RUNNING: u8 = 0;
const CLOSING: u8 = 1;
const READY_TO_EXIT: u8 = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ShutdownPhase {
    Running,
    Closing,
    ReadyToExit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum BeginClosing {
    Started,
    AlreadyClosing,
    Ready,
}

pub(super) struct ShutdownState {
    phase: AtomicU8,
}

impl ShutdownState {
    pub(super) const fn new() -> Self {
        Self {
            phase: AtomicU8::new(RUNNING),
        }
    }

    pub(super) fn phase(&self) -> ShutdownPhase {
        match self.phase.load(Ordering::Acquire) {
            RUNNING => ShutdownPhase::Running,
            CLOSING => ShutdownPhase::Closing,
            _ => ShutdownPhase::ReadyToExit,
        }
    }

    pub(super) fn begin_closing(&self) -> BeginClosing {
        match self
            .phase
            .compare_exchange(RUNNING, CLOSING, Ordering::AcqRel, Ordering::Acquire)
        {
            Ok(_) => BeginClosing::Started,
            Err(CLOSING) => BeginClosing::AlreadyClosing,
            Err(_) => BeginClosing::Ready,
        }
    }

    pub(super) fn mark_ready(&self) -> bool {
        self.phase
            .compare_exchange(CLOSING, READY_TO_EXIT, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }
}

impl Default for ShutdownState {
    fn default() -> Self {
        Self::new()
    }
}
