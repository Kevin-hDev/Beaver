use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum HostExitKind {
    Requested,
    Unexpected,
}

#[derive(Debug)]
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
        // Un arrêt concurrent ne doit jamais dégrader un redémarrage déjà demandé.
        self.restarting.fetch_or(restarting, Ordering::AcqRel);
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
