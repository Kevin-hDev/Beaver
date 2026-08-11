use super::super::constants::CEF_TRACKER_DROP_TIMEOUT;
use super::super::CefUnavailableCategory;
use super::tracker::{failure_from_id, MacCefTracker, MacTrackerShared};
use std::sync::atomic::Ordering;
#[cfg(test)]
use std::time::Duration;
use std::time::Instant;

impl MacTrackerShared {
    pub(super) fn fail(&self, category: CefUnavailableCategory) {
        if self
            .failure
            .compare_exchange(0, category.id(), Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            ::log::error!("[browser] macOS supervision failed ({})", category.code());
            if let Some(app) = &self.shutdown_app {
                crate::app_exit::request(app, 1);
            }
        }
    }

    pub(super) fn failure(&self) -> Option<CefUnavailableCategory> {
        failure_from_id(self.failure.load(Ordering::Acquire))
    }

    pub(in crate::services::browser) fn emergency_close(
        &self,
        admission_deadline: Instant,
        helper_exit_deadline: Instant,
    ) -> bool {
        let deadline_ticks = super::clock::ticks_at(helper_exit_deadline);
        let gate_closed = self.gate.close_and_wait(admission_deadline);
        let table_closed = self.table.close_and_invalidate(admission_deadline);
        let signaled = deadline_ticks.is_ok_and(|deadline| {
            let pending = self.pending.begin_closing(deadline).is_ok();
            let admitted = self.emergency.begin_closing(deadline).is_ok();
            pending && admitted
        });
        if !signaled {
            self.fail(CefUnavailableCategory::Reaper);
        }
        gate_closed && table_closed
    }

    pub(in crate::services::browser) fn emergency_force(&self) {
        if !self.reaper_control.force() {
            self.fail(CefUnavailableCategory::Reaper);
        }
    }

    pub(in crate::services::browser) fn emergency_has_runnable(&self) -> bool {
        self.emergency.has_entries()
    }
}

#[cfg(test)]
impl MacCefTracker {
    pub(in crate::services::browser) fn close_gate_for_test(&self) -> bool {
        let now = Instant::now();
        self.shared
            .emergency_close(now + CEF_TRACKER_DROP_TIMEOUT, now + Duration::from_secs(2))
    }

    pub(in crate::services::browser) fn force_for_test(&self) {
        self.shared.emergency_force();
    }

    pub(in crate::services::browser) fn stop_normal_for_test(&mut self) {
        self.shared.tracker_stopping.store(true, Ordering::Release);
        if let Some(thread) = self.normal_thread.take() {
            thread.join().expect("normal tracker thread");
        }
    }
}

impl Drop for MacCefTracker {
    fn drop(&mut self) {
        let now = Instant::now();
        let _ = self.shared.emergency_close(
            now + CEF_TRACKER_DROP_TIMEOUT,
            now + CEF_TRACKER_DROP_TIMEOUT,
        );
        self.shared.tracker_stopping.store(true, Ordering::Release);
        if self
            .normal_thread
            .take()
            .is_some_and(|thread| thread.join().is_err())
        {
            self.shared.fail(CefUnavailableCategory::Reaper);
        }
        self.shared.emergency_force();
    }
}
