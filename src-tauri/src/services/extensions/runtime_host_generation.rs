use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum HostExitKind {
    Requested,
    Unexpected,
}

pub(super) struct HostGeneration {
    pub(super) number: u64,
    pub(super) stopping: AtomicBool,
    restarting: AtomicBool,
}

impl HostGeneration {
    pub(super) fn new(number: u64) -> Self {
        Self {
            number,
            stopping: AtomicBool::new(false),
            restarting: AtomicBool::new(false),
        }
    }

    pub(super) fn begin_stop(&self, restarting: bool) {
        self.restarting.store(restarting, Ordering::Release);
        self.stopping.store(true, Ordering::Release);
    }

    pub(super) fn is_stopping(&self) -> bool {
        self.stopping.load(Ordering::Acquire)
    }

    #[cfg(test)]
    pub(super) fn is_restarting(&self) -> bool {
        self.restarting.load(Ordering::Acquire)
    }

    pub(super) fn exit_kind(&self) -> HostExitKind {
        if self.is_stopping() {
            HostExitKind::Requested
        } else {
            HostExitKind::Unexpected
        }
    }
}
