use super::super::gate::CefLaunchGate;
use super::super::{CefAuthorityTable, CefIpcNames, CefProcessRole, CefUnavailableCategory};
use super::native_authority::WindowsNativeAuthority;
use super::objects::WindowsPublicationObjects;
use super::ticket::WindowsLaunchTicket;
use super::tracker_loop::run_tracker;
use super::tracker_pending::{WindowsPendingLaunch, WindowsPendingSlots};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

pub(super) struct WindowsTrackerShared {
    pub(super) table: CefAuthorityTable,
    pub(super) native: Arc<WindowsNativeAuthority>,
    pub(super) pending: WindowsPendingSlots,
    pub(super) gate: CefLaunchGate,
    pub(super) stopping: AtomicBool,
    failure: AtomicU8,
    pub(super) expected_executable: std::path::PathBuf,
    pub(super) parent_pid: u32,
}

pub(in crate::services::browser) struct WindowsCefTracker {
    shared: Arc<WindowsTrackerShared>,
    thread: Option<JoinHandle<()>>,
}

impl WindowsCefTracker {
    pub(in crate::services::browser) fn start(
        expected_executable: &Path,
    ) -> Result<Self, CefUnavailableCategory> {
        let expected_executable = super::process_query::canonical_executable(expected_executable)
            .map_err(|_| CefUnavailableCategory::Reaper)?;
        let shared = Arc::new(WindowsTrackerShared {
            table: CefAuthorityTable::new(),
            native: WindowsNativeAuthority::new(),
            pending: WindowsPendingSlots::new(),
            gate: CefLaunchGate::new(),
            stopping: AtomicBool::new(false),
            failure: AtomicU8::new(0),
            expected_executable,
            parent_pid: std::process::id(),
        });
        let worker = Arc::clone(&shared);
        let thread = std::thread::Builder::new()
            .name("cef-windows-tracker".to_string())
            .spawn(move || run_tracker(worker))
            .map_err(|_| CefUnavailableCategory::Reaper)?;
        Ok(Self {
            shared,
            thread: Some(thread),
        })
    }

    pub(in crate::services::browser) fn reserve(
        &self,
    ) -> Result<WindowsLaunchTicket, CefUnavailableCategory> {
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
        let objects = WindowsPublicationObjects::create(&names, reservation.marker().generation())?;
        let ticket = WindowsLaunchTicket::new(reservation.marker());
        let slot = reservation.marker().slot();
        if self.shared.gate.is_closed() {
            return Err(CefUnavailableCategory::Admission);
        }
        self.shared.pending.install(
            slot,
            WindowsPendingLaunch {
                reservation,
                objects,
            },
        )?;
        Ok(ticket)
    }

    pub(in crate::services::browser) fn failure(&self) -> Option<CefUnavailableCategory> {
        failure_from_id(self.shared.failure.load(Ordering::Acquire))
    }
}

impl WindowsTrackerShared {
    pub(super) fn fail(&self, category: CefUnavailableCategory) {
        let _ =
            self.failure
                .compare_exchange(0, category.id(), Ordering::AcqRel, Ordering::Acquire);
    }

    pub(super) fn failure(&self) -> Option<CefUnavailableCategory> {
        failure_from_id(self.failure.load(Ordering::Acquire))
    }
}

impl Drop for WindowsCefTracker {
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
