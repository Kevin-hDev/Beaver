use super::super::constants::CEF_TRACKER_DROP_TIMEOUT;
use super::super::gate::CefLaunchGate;
use super::super::{CefAuthorityTable, CefLaunchTicket, CefUnavailableCategory};
use super::native_authority::WindowsNativeAuthority;
use super::tracker_loop::run_tracker;
use super::tracker_pending::WindowsPendingSlots;
use crate::services::browser::cef_preflight::CefPreflightError;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Instant;

#[cfg(test)]
mod test_api;

pub(in crate::services::browser) struct WindowsTrackerShared {
    pub(super) table: CefAuthorityTable,
    pub(super) native: Arc<WindowsNativeAuthority>,
    pub(super) pending: WindowsPendingSlots,
    pub(super) gate: CefLaunchGate,
    pub(super) stopping: AtomicBool,
    failure: AtomicU8,
    pub(super) expected_executable: std::path::PathBuf,
    pub(super) parent_pid: u32,
    shutdown_app: Option<tauri::AppHandle>,
}

pub(in crate::services::browser) struct WindowsCefTracker {
    shared: Arc<WindowsTrackerShared>,
    thread: Option<JoinHandle<()>>,
}

#[derive(Clone)]
pub(in crate::services::browser) struct WindowsCefTrackerHandle {
    pub(super) shared: Arc<WindowsTrackerShared>,
}

impl WindowsCefTracker {
    #[cfg(test)]
    pub(in crate::services::browser) fn start(
        expected_executable: &Path,
    ) -> Result<Self, CefUnavailableCategory> {
        Self::start_inner(expected_executable, None).map_err(CefPreflightError::category)
    }

    pub(in crate::services::browser) fn start_supervised(
        expected_executable: &Path,
        app: tauri::AppHandle,
    ) -> Result<Self, CefPreflightError> {
        let tracker = Self::start_inner(expected_executable, Some(app))?;
        super::super::emergency::register_windows(Arc::clone(&tracker.shared))
            .map_err(|_| CefPreflightError::deterministic(CefUnavailableCategory::Reaper))?;
        Ok(tracker)
    }

    fn start_inner(
        expected_executable: &Path,
        shutdown_app: Option<tauri::AppHandle>,
    ) -> Result<Self, CefPreflightError> {
        let expected_executable =
            super::process_query::canonical_executable(expected_executable)
                .map_err(|_| CefPreflightError::deterministic(CefUnavailableCategory::Reaper))?;
        let shared = Arc::new(WindowsTrackerShared {
            table: CefAuthorityTable::new(),
            native: WindowsNativeAuthority::new(),
            pending: WindowsPendingSlots::new(),
            gate: CefLaunchGate::new(),
            stopping: AtomicBool::new(false),
            failure: AtomicU8::new(0),
            expected_executable,
            parent_pid: std::process::id(),
            shutdown_app,
        });
        let worker = Arc::clone(&shared);
        let failure = Arc::clone(&shared);
        let thread = std::thread::Builder::new()
            .name("cef-windows-tracker".to_string())
            .spawn(move || {
                if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run_tracker(worker)))
                    .is_err()
                {
                    failure.fail(CefUnavailableCategory::Reaper);
                }
            })
            .map_err(|error| CefPreflightError::from_io(CefUnavailableCategory::Reaper, &error))?;
        Ok(Self {
            shared,
            thread: Some(thread),
        })
    }

    #[cfg(test)]
    pub(in crate::services::browser) fn reserve(
        &self,
    ) -> Result<CefLaunchTicket, CefUnavailableCategory> {
        self.handle().reserve()
    }

    pub(in crate::services::browser) fn handle(&self) -> WindowsCefTrackerHandle {
        WindowsCefTrackerHandle {
            shared: Arc::clone(&self.shared),
        }
    }

    #[cfg(test)]
    pub(in crate::services::browser) fn failure(&self) -> Option<CefUnavailableCategory> {
        self.shared.failure()
    }
}

impl WindowsTrackerShared {
    pub(super) fn fail(&self, category: CefUnavailableCategory) {
        if self
            .failure
            .compare_exchange(0, category.id(), Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            if let Some(app) = &self.shutdown_app {
                crate::app_exit::request(app, 1);
            }
        }
    }

    pub(super) fn failure(&self) -> Option<CefUnavailableCategory> {
        failure_from_id(self.failure.load(Ordering::Acquire))
    }

    pub(in crate::services::browser) fn emergency_close(&self, deadline: Instant) -> bool {
        let gate_closed = self.gate.close_and_wait(deadline);
        let table_closed = self.table.close_and_invalidate(deadline);
        gate_closed && table_closed
    }

    pub(in crate::services::browser) fn emergency_force(&self) {
        if self.native.force_all().is_err() {
            self.fail(CefUnavailableCategory::Reaper);
        }
    }

    pub(in crate::services::browser) fn emergency_has_runnable(&self) -> bool {
        self.native.occupied_slots() != 0
    }
}

impl Drop for WindowsCefTracker {
    fn drop(&mut self) {
        let deadline = Instant::now() + CEF_TRACKER_DROP_TIMEOUT;
        let _ = self.shared.gate.close_and_wait(deadline);
        let _ = self.shared.table.close_and_invalidate(deadline);
        self.shared.stopping.store(true, Ordering::Release);
        if self
            .thread
            .take()
            .is_some_and(|thread| thread.join().is_err())
        {
            self.shared.fail(CefUnavailableCategory::Reaper);
        }
    }
}

fn failure_from_id(value: u8) -> Option<CefUnavailableCategory> {
    if value == 0 {
        None
    } else {
        Some(CefUnavailableCategory::from_id(value).unwrap_or(CefUnavailableCategory::Reaper))
    }
}

impl std::fmt::Debug for WindowsCefTracker {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("WindowsCefTracker([redacted])")
    }
}

impl std::fmt::Debug for WindowsCefTrackerHandle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("WindowsCefTrackerHandle([redacted])")
    }
}
