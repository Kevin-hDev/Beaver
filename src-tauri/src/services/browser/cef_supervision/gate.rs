use super::constants::GATE_RECHECK;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

const CLOSED_BIT: u64 = 1 << 63;
const PERMIT_MASK: u64 = !CLOSED_BIT;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CefGateError {
    Closed,
}

pub(in crate::services::browser) struct CefLaunchGate {
    state: AtomicU64,
}

impl CefLaunchGate {
    pub(super) const fn new() -> Self {
        Self {
            state: AtomicU64::new(0),
        }
    }

    pub(super) fn try_enter(&self) -> Result<CefGatePermit<'_>, CefGateError> {
        let mut current = self.state.load(Ordering::Acquire);
        loop {
            if current & CLOSED_BIT != 0 || current & PERMIT_MASK == PERMIT_MASK {
                return Err(CefGateError::Closed);
            }
            match self.state.compare_exchange_weak(
                current,
                current + 1,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return Ok(CefGatePermit { gate: self }),
                Err(observed) => current = observed,
            }
        }
    }

    pub(super) fn is_closed(&self) -> bool {
        self.state.load(Ordering::Acquire) & CLOSED_BIT != 0
    }

    pub(super) fn close_and_wait(&self, deadline: Instant) -> bool {
        self.state.fetch_or(CLOSED_BIT, Ordering::AcqRel);
        loop {
            if self.state.load(Ordering::Acquire) & PERMIT_MASK == 0 {
                return true;
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return false;
            }
            std::thread::park_timeout(remaining.min(GATE_RECHECK));
        }
    }
}

impl Default for CefLaunchGate {
    fn default() -> Self {
        Self::new()
    }
}

pub(super) struct CefGatePermit<'a> {
    gate: &'a CefLaunchGate,
}

impl Drop for CefGatePermit<'_> {
    fn drop(&mut self) {
        self.gate.state.fetch_sub(1, Ordering::AcqRel);
    }
}
