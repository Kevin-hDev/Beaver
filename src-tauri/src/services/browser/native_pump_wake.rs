use super::{native_pump::PumpTarget, pump_gate::PumpGate};
use dispatch2::MainThreadBound;
use objc2::{rc::Retained, MainThreadMarker};
use objc2_foundation::NSThread;
use std::sync::Arc;

#[derive(Clone)]
pub(super) struct PumpWake {
    gate: Arc<PumpGate>,
    _objects: Arc<MainThreadBound<PumpObjects>>,
    owner_thread: usize,
    target: usize,
}

struct PumpObjects {
    _owner_thread: Retained<NSThread>,
    _target: Retained<PumpTarget>,
}

impl PumpWake {
    pub(super) fn new(
        gate: Arc<PumpGate>,
        owner_thread: Retained<NSThread>,
        target: Retained<PumpTarget>,
        marker: MainThreadMarker,
    ) -> Self {
        Self {
            owner_thread: Retained::as_ptr(&owner_thread) as usize,
            target: Retained::as_ptr(&target) as usize,
            _objects: Arc::new(MainThreadBound::new(
                PumpObjects {
                    _owner_thread: owner_thread,
                    _target: target,
                },
                marker,
            )),
            gate,
        }
    }

    pub(super) fn notify(&self, delay_ms: i64) {
        if self.gate.request() {
            self.queue(delay_ms);
        }
    }

    pub(super) fn start(&self) {
        let _ = self.gate.request();
        self.queue(0);
    }

    fn queue(&self, delay_ms: i64) {
        // MainThreadBound owns both Objective-C objects until this wake handle
        // is dropped. Apple permits performSelector:onThread: from any thread.
        let target = unsafe { &*(self.target as *const PumpTarget) };
        let owner_thread = unsafe { &*(self.owner_thread as *const NSThread) };
        super::native_pump::queue_on_thread(target, owner_thread, delay_ms);
    }
}
