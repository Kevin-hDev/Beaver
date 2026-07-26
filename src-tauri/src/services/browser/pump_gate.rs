use std::sync::atomic::{AtomicU8, Ordering};

const IDLE: u8 = 0;
const REQUESTED: u8 = 1;
const RUNNING: u8 = 2;
const RERUN: u8 = 3;
const STOPPED: u8 = 4;

#[derive(Default)]
pub(super) struct PumpGate {
    state: AtomicU8,
}

impl PumpGate {
    pub(super) fn request(&self) -> bool {
        loop {
            match self.state.load(Ordering::Acquire) {
                IDLE => {
                    if self
                        .state
                        .compare_exchange(IDLE, REQUESTED, Ordering::AcqRel, Ordering::Acquire)
                        .is_ok()
                    {
                        return true;
                    }
                }
                RUNNING => {
                    if self
                        .state
                        .compare_exchange(RUNNING, RERUN, Ordering::AcqRel, Ordering::Acquire)
                        .is_ok()
                    {
                        return false;
                    }
                }
                REQUESTED | RERUN | STOPPED => return false,
                _ => return false,
            }
        }
    }

    pub(super) fn begin_dispatch(&self) -> bool {
        self.state
            .compare_exchange(REQUESTED, RUNNING, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    pub(super) fn complete_and_requeue(&self) -> bool {
        loop {
            match self.state.load(Ordering::Acquire) {
                RUNNING => {
                    if self
                        .state
                        .compare_exchange(RUNNING, IDLE, Ordering::AcqRel, Ordering::Acquire)
                        .is_ok()
                    {
                        return false;
                    }
                }
                RERUN => {
                    if self
                        .state
                        .compare_exchange(RERUN, REQUESTED, Ordering::AcqRel, Ordering::Acquire)
                        .is_ok()
                    {
                        return true;
                    }
                }
                IDLE | REQUESTED | STOPPED => return false,
                _ => return false,
            }
        }
    }

    pub(super) fn stop(&self) {
        self.state.store(STOPPED, Ordering::Release);
    }

    pub(super) fn is_stopped(&self) -> bool {
        self.state.load(Ordering::Acquire) == STOPPED
    }
}
