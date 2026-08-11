use super::super::constants::{CEF_REAPER_START_TIMEOUT, CEF_TRACKER_POLL};
use super::super::mac_supervision_failure::MacSupervisionFailure;
use super::super::CefUnavailableCategory;
use super::tracker::MacTrackerShared;
use crate::services::browser::cef_preflight::CefPreflightError;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;

pub(super) struct MacReaperControl {
    force_requested: AtomicBool,
    stopping: AtomicBool,
    healthy: AtomicBool,
}

pub(super) struct MacEmergencyReaper {
    control: Arc<MacReaperControl>,
    thread: Option<JoinHandle<()>>,
}

impl MacReaperControl {
    pub(super) fn new() -> Self {
        Self {
            force_requested: AtomicBool::new(false),
            stopping: AtomicBool::new(false),
            healthy: AtomicBool::new(false),
        }
    }

    pub(super) fn force(&self) -> bool {
        self.force_requested.store(true, Ordering::Release);
        self.healthy.load(Ordering::Acquire)
    }

    fn stop(&self) {
        self.stopping.store(true, Ordering::Release);
    }
}

impl MacEmergencyReaper {
    pub(super) fn start(shared: Arc<MacTrackerShared>) -> Result<Self, CefPreflightError> {
        let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel(1);
        let control = Arc::clone(&shared.reaper_control);
        let worker_control = Arc::clone(&control);
        let thread = std::thread::Builder::new()
            .name("cef-macos-emergency-reaper".to_string())
            .spawn(move || {
                worker_control.healthy.store(true, Ordering::Release);
                let _ = ready_tx.send(());
                let panicked =
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(&shared)))
                        .is_err();
                worker_control.healthy.store(false, Ordering::Release);
                if panicked && !worker_control.stopping.load(Ordering::Acquire) {
                    shared.fail(MacSupervisionFailure::EmergencyReaperPanic);
                }
            })
            .map_err(|error| CefPreflightError::from_io(CefUnavailableCategory::Reaper, &error))?;
        if ready_rx.recv_timeout(CEF_REAPER_START_TIMEOUT).is_err()
            || !control.healthy.load(Ordering::Acquire)
        {
            control.stop();
            drop(thread);
            return Err(CefPreflightError::deterministic(
                CefUnavailableCategory::Reaper,
            ));
        }
        Ok(Self {
            control,
            thread: Some(thread),
        })
    }
}

fn run(shared: &MacTrackerShared) {
    while !shared.reaper_control.stopping.load(Ordering::Acquire) {
        if shared
            .reaper_control
            .force_requested
            .load(Ordering::Acquire)
            && shared.emergency.force_pass().is_err()
        {
            shared.fail(MacSupervisionFailure::ForcePass);
        }
        std::thread::park_timeout(CEF_TRACKER_POLL);
    }
    if shared
        .reaper_control
        .force_requested
        .load(Ordering::Acquire)
    {
        let _ = shared.emergency.force_pass();
    }
}

impl Drop for MacEmergencyReaper {
    fn drop(&mut self) {
        self.control.stop();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
