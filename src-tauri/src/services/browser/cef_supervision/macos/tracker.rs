use super::super::gate::CefLaunchGate;
use super::super::{
    CefAuthorityTable, CefIpcNames, CefLaunchTicket, CefProcessRole, CefUnavailableCategory,
};
use super::pending::{MacPendingLaunch, MacPendingSlots};
use super::{MacEmergencyReaper, MacEmergencySlots, MacPublicationObjects, MacReaperControl};
use crate::services::browser::cef_preflight::CefPreflightError;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;

pub(in crate::services::browser) struct MacTrackerShared {
    pub(super) table: CefAuthorityTable,
    pub(super) pending: MacPendingSlots,
    pub(super) gate: CefLaunchGate,
    pub(super) tracker_stopping: AtomicBool,
    pub(super) failure: AtomicU8,
    pub(super) expected_executable: PathBuf,
    pub(super) parent_pid: u32,
    pub(super) root: PathBuf,
    pub(super) shutdown_app: Option<tauri::AppHandle>,
    pub(super) emergency: Arc<MacEmergencySlots>,
    pub(super) reaper_control: Arc<MacReaperControl>,
}

pub(in crate::services::browser) struct MacCefTracker {
    pub(super) shared: Arc<MacTrackerShared>,
    pub(super) normal_thread: Option<JoinHandle<()>>,
    _emergency_reaper: MacEmergencyReaper,
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
        Self::start_inner(expected_executable, root, None).map_err(CefPreflightError::category)
    }

    pub(in crate::services::browser) fn start_supervised(
        expected_executable: &Path,
        root: PathBuf,
        app: tauri::AppHandle,
    ) -> Result<Self, CefPreflightError> {
        let tracker = Self::start_inner(expected_executable, root, Some(app))?;
        super::super::emergency::register_macos(Arc::clone(&tracker.shared))
            .map_err(|_| CefPreflightError::deterministic(CefUnavailableCategory::Reaper))?;
        Ok(tracker)
    }

    fn start_inner(
        expected_executable: &Path,
        root: PathBuf,
        shutdown_app: Option<tauri::AppHandle>,
    ) -> Result<Self, CefPreflightError> {
        let expected_executable = dunce::canonicalize(expected_executable)
            .map_err(|error| CefPreflightError::from_io(CefUnavailableCategory::Reaper, &error))?;
        let emergency = Arc::new(MacEmergencySlots::new());
        let reaper_control = Arc::new(MacReaperControl::new());
        let shared = Arc::new(MacTrackerShared {
            table: CefAuthorityTable::new(),
            pending: MacPendingSlots::new(),
            gate: CefLaunchGate::new(),
            tracker_stopping: AtomicBool::new(false),
            failure: AtomicU8::new(0),
            expected_executable,
            parent_pid: std::process::id(),
            root,
            shutdown_app,
            emergency,
            reaper_control,
        });
        let emergency_reaper = MacEmergencyReaper::start(Arc::clone(&shared))?;
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
            .map_err(|error| CefPreflightError::from_io(CefUnavailableCategory::Reaper, &error))?;
        Ok(Self {
            shared,
            normal_thread: Some(thread),
            _emergency_reaper: emergency_reaper,
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
        if self.failure().is_some() || self.shared.tracker_stopping.load(Ordering::Acquire) {
            return Err(CefUnavailableCategory::Admission);
        }
        let reservation = self
            .shared
            .table
            .try_reserve(CefProcessRole::Helper)
            .map_err(|_| CefUnavailableCategory::Admission)?;
        let names = CefIpcNames::from_marker(reservation.marker())
            .map_err(|_| CefUnavailableCategory::Object)?;
        let objects = Arc::new(MacPublicationObjects::create(
            &self.shared.root,
            &names,
            reservation.marker().generation(),
        )?);
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

pub(super) fn failure_from_id(value: u8) -> Option<CefUnavailableCategory> {
    (value != 0)
        .then(|| CefUnavailableCategory::from_id(value).unwrap_or(CefUnavailableCategory::Reaper))
}
