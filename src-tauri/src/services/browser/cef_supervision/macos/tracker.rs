use super::super::gate::CefLaunchGate;
use super::super::{
    CefAuthorityTable, CefIpcNames, CefLaunchTicket, CefProcessRole, CefUnavailableCategory,
};
use super::pending::{MacPendingLaunch, MacPendingSlots};
use super::MacPublicationObjects;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

pub(in crate::services::browser) struct MacTrackerShared {
    pub(super) table: CefAuthorityTable,
    pub(super) pending: MacPendingSlots,
    pub(super) gate: CefLaunchGate,
    pub(super) stopping: AtomicBool,
    failure: AtomicU8,
    pub(super) expected_executable: PathBuf,
    pub(super) parent_pid: u32,
    pub(super) root: PathBuf,
    shutdown_app: Option<tauri::AppHandle>,
    pub(super) force_requested: AtomicBool,
    pub(super) active_count: AtomicUsize,
}

pub(in crate::services::browser) struct MacCefTracker {
    shared: Arc<MacTrackerShared>,
    thread: Option<JoinHandle<()>>,
}

#[derive(Clone)]
pub(in crate::services::browser) struct MacCefTrackerHandle {
    shared: Arc<MacTrackerShared>,
}

impl std::fmt::Debug for MacCefTrackerHandle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("MacCefTrackerHandle([redacted])")
    }
}

impl MacCefTracker {
    #[cfg(test)]
    pub(in crate::services::browser) fn start(
        expected_executable: &Path,
        root: PathBuf,
    ) -> Result<Self, CefUnavailableCategory> {
        Self::start_inner(expected_executable, root, None)
    }

    pub(in crate::services::browser) fn start_supervised(
        expected_executable: &Path,
        root: PathBuf,
        app: tauri::AppHandle,
    ) -> Result<Self, CefUnavailableCategory> {
        let tracker = Self::start_inner(expected_executable, root, Some(app))?;
        super::super::emergency::register_macos(Arc::clone(&tracker.shared))
            .map_err(|_| CefUnavailableCategory::Reaper)?;
        Ok(tracker)
    }

    fn start_inner(
        expected_executable: &Path,
        root: PathBuf,
        shutdown_app: Option<tauri::AppHandle>,
    ) -> Result<Self, CefUnavailableCategory> {
        let expected_executable =
            dunce::canonicalize(expected_executable).map_err(|_| CefUnavailableCategory::Reaper)?;
        let shared = Arc::new(MacTrackerShared {
            table: CefAuthorityTable::new(),
            pending: MacPendingSlots::new(),
            gate: CefLaunchGate::new(),
            stopping: AtomicBool::new(false),
            failure: AtomicU8::new(0),
            expected_executable,
            parent_pid: std::process::id(),
            root,
            shutdown_app,
            force_requested: AtomicBool::new(false),
            active_count: AtomicUsize::new(0),
        });
        let worker = Arc::clone(&shared);
        let failure = Arc::clone(&shared);
        let thread = std::thread::Builder::new()
            .name("cef-macos-reaper".to_string())
            .spawn(move || {
                if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    super::tracker_loop::run_tracker(worker)
                }))
                .is_err()
                {
                    failure.fail(CefUnavailableCategory::Reaper);
                }
            })
            .map_err(|_| CefUnavailableCategory::Reaper)?;
        Ok(Self {
            shared,
            thread: Some(thread),
        })
    }

    pub(in crate::services::browser) fn handle(&self) -> MacCefTrackerHandle {
        MacCefTrackerHandle {
            shared: Arc::clone(&self.shared),
        }
    }
}

impl MacCefTrackerHandle {
    pub(in crate::services::browser) fn reserve(
        &self,
    ) -> Result<CefLaunchTicket, CefUnavailableCategory> {
        let _permit = self
            .shared
            .gate
            .try_enter()
            .map_err(|_| CefUnavailableCategory::Admission)?;
        if self.failure().is_some() || self.shared.stopping.load(Ordering::Acquire) {
            return Err(CefUnavailableCategory::Admission);
        }
        let reservation = self
            .shared
            .table
            .try_reserve(CefProcessRole::Helper)
            .map_err(|_| CefUnavailableCategory::Admission)?;
        let names = CefIpcNames::from_marker(reservation.marker())
            .map_err(|_| CefUnavailableCategory::Object)?;
        let objects = MacPublicationObjects::create(
            &self.shared.root,
            &names,
            reservation.marker().generation(),
        )?;
        let ticket = CefLaunchTicket::new(reservation.marker());
        let slot = reservation.marker().slot();
        if self.shared.gate.is_closed() {
            return Err(CefUnavailableCategory::Admission);
        }
        self.shared.pending.install(
            slot,
            MacPendingLaunch {
                reservation,
                objects,
            },
        )?;
        Ok(ticket)
    }

    pub(in crate::services::browser) fn fail(&self, category: CefUnavailableCategory) {
        self.shared.fail(category);
    }

    fn failure(&self) -> Option<CefUnavailableCategory> {
        failure_from_id(self.shared.failure.load(Ordering::Acquire))
    }
}

impl MacTrackerShared {
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
        self.force_requested.store(true, Ordering::Release);
    }

    pub(in crate::services::browser) fn emergency_has_runnable(&self) -> bool {
        self.active_count.load(Ordering::Acquire) != 0
    }
}

#[cfg(test)]
impl MacCefTracker {
    pub(in crate::services::browser) fn close_gate_for_test(&self) -> bool {
        self.shared
            .emergency_close(Instant::now() + Duration::from_millis(50))
    }

    pub(in crate::services::browser) fn force_for_test(&self) {
        self.shared.emergency_force();
    }
}

impl Drop for MacCefTracker {
    fn drop(&mut self) {
        let deadline = Instant::now() + Duration::from_millis(50);
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
    (value != 0)
        .then(|| CefUnavailableCategory::from_id(value).unwrap_or(CefUnavailableCategory::Reaper))
}
