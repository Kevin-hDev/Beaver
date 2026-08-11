use super::super::gate::CefLaunchGate;
use super::super::mac_supervision_failure::MacSupervisionFailure;
use super::super::{CefAuthorityTable, CefUnavailableCategory};
use super::emergency_slots::MacEmergencySlots;
use super::pending::MacPendingSlots;
use super::reaper::{MacEmergencyReaper, MacReaperControl};
use crate::services::browser::cef_preflight::CefPreflightError;
use crate::services::browser::native_paths::MacHelperExecutables;
#[cfg(test)]
use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU8};
use std::sync::Arc;
use std::thread::JoinHandle;

pub(in crate::services::browser) struct MacTrackerShared {
    pub(super) table: CefAuthorityTable,
    pub(super) pending: MacPendingSlots,
    pub(super) gate: CefLaunchGate,
    pub(super) tracker_stopping: AtomicBool,
    pub(super) failure: AtomicU8,
    pub(super) expected_executables: MacHelperExecutables,
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
    pub(super) shared: Arc<MacTrackerShared>,
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
        let expected_executables = std::array::from_fn(|_| expected_executable.to_path_buf());
        Self::start_inner(&expected_executables, root, None).map_err(CefPreflightError::category)
    }

    pub(in crate::services::browser) fn start_supervised(
        expected_executables: &MacHelperExecutables,
        root: PathBuf,
        app: tauri::AppHandle,
    ) -> Result<Self, CefPreflightError> {
        let tracker = Self::start_inner(expected_executables, root, Some(app))?;
        super::super::emergency::register_macos(Arc::clone(&tracker.shared))
            .map_err(|_| CefPreflightError::deterministic(CefUnavailableCategory::Reaper))?;
        Ok(tracker)
    }

    fn start_inner(
        expected_executables: &MacHelperExecutables,
        root: PathBuf,
        shutdown_app: Option<tauri::AppHandle>,
    ) -> Result<Self, CefPreflightError> {
        let mut expected_executables = expected_executables.clone();
        for executable in &mut expected_executables {
            *executable = dunce::canonicalize(&*executable).map_err(|error| {
                CefPreflightError::from_io(CefUnavailableCategory::Reaper, &error)
            })?;
        }
        let emergency = Arc::new(MacEmergencySlots::new());
        let reaper_control = Arc::new(MacReaperControl::new());
        let shared = Arc::new(MacTrackerShared {
            table: CefAuthorityTable::new(),
            pending: MacPendingSlots::new(),
            gate: CefLaunchGate::new(),
            tracker_stopping: AtomicBool::new(false),
            failure: AtomicU8::new(0),
            expected_executables,
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
                    failure.fail(MacSupervisionFailure::TrackerPanic);
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

pub(super) fn failure_from_id(value: u8) -> Option<CefUnavailableCategory> {
    (value != 0)
        .then(|| CefUnavailableCategory::from_id(value).unwrap_or(CefUnavailableCategory::Reaper))
}
