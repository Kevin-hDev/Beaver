use super::super::constants::CEF_TRACKER_DROP_TIMEOUT;
use super::super::mac_supervision_failure::MacSupervisionFailure;
use super::super::CefUnavailableCategory;
use super::tracker::{failure_from_id, MacCefTracker, MacTrackerShared};
use std::sync::atomic::Ordering;
#[cfg(test)]
use std::time::Duration;
use std::time::Instant;

impl MacTrackerShared {
    pub(super) fn fail(&self, failure: MacSupervisionFailure) {
        let category = failure.category();
        if self
            .failure
            .compare_exchange(0, category.id(), Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            ::log::error!(
                "[browser] macOS supervision failed ({}) reason={}",
                category.code(),
                failure.code()
            );
            #[cfg(feature = "e2e")]
            eprintln!("[e2e-supervision-failure] {}", failure.code());
            if let Some(app) = &self.shutdown_app {
                crate::services::e2e_profile::report_browser_exit_source(
                    crate::services::e2e_profile::BrowserExitSource::Supervision,
                );
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
        let failure = match deadline_ticks {
            Err(_) => Some(MacSupervisionFailure::ClosingClock),
            Ok(deadline) => {
                let pending_failed = self.pending.begin_closing(deadline).is_err();
                let admitted_failed = self.emergency.begin_closing(deadline).is_err();
                if pending_failed {
                    Some(MacSupervisionFailure::PendingCloseSignal)
                } else if admitted_failed {
                    Some(MacSupervisionFailure::AdmittedCloseSignal)
                } else {
                    None
                }
            }
        };
        if let Some(failure) = failure {
            self.fail(failure);
        }
        gate_closed && table_closed
    }

    pub(in crate::services::browser) fn emergency_force(&self) {
        if !self.reaper_control.force() {
            self.fail(MacSupervisionFailure::EmergencyUnavailable);
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

    pub(in crate::services::browser) fn failure_for_test(&self) -> Option<CefUnavailableCategory> {
        self.shared.failure()
    }

    pub(in crate::services::browser) fn has_runnable_for_test(&self) -> bool {
        self.shared.emergency_has_runnable()
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
            self.shared.fail(MacSupervisionFailure::TrackerJoinPanic);
        }
        self.shared.emergency_force();
    }
}
