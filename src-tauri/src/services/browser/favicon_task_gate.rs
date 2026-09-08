use std::sync::atomic::{AtomicBool, Ordering};

// Each posted task owns its reservation, even if CEF drops it without execution.
pub(super) struct TaskReservation<'a> {
    gate: &'a AtomicBool,
    released: AtomicBool,
}
impl<'a> TaskReservation<'a> {
    pub(super) fn acquire(gate: &'a AtomicBool) -> Option<Self> {
        if gate.swap(true, Ordering::AcqRel) {
            return None;
        }
        Some(Self {
            gate,
            released: AtomicBool::new(false),
        })
    }
    pub(super) fn release(&self) {
        if !self.released.swap(true, Ordering::AcqRel) {
            self.gate.store(false, Ordering::Release);
        }
    }
}
impl Drop for TaskReservation<'_> {
    fn drop(&mut self) {
        self.release();
    }
}
