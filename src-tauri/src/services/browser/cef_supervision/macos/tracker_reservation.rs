use super::super::constants::publication_deadline;
use super::super::mac_supervision_failure::MacSupervisionFailure;
use super::super::{CefIpcNames, CefLaunchTicket, CefProcessRole, CefUnavailableCategory};
use super::pending::MacPendingLaunch;
use super::tracker::{failure_from_id, MacCefTrackerHandle};
use super::MacPublicationObjects;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Instant;

impl MacCefTrackerHandle {
    pub(in crate::services::browser) fn reserve(
        &self,
    ) -> Result<CefLaunchTicket, CefUnavailableCategory> {
        self.reserve_until(publication_deadline(Instant::now()))
    }

    fn reserve_until(
        &self,
        expires_at: Instant,
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
                expires_at,
            },
        )?;
        Ok(ticket)
    }

    #[cfg(test)]
    pub(in crate::services::browser) fn reserve_until_for_test(
        &self,
        expires_at: Instant,
    ) -> Result<CefLaunchTicket, CefUnavailableCategory> {
        self.reserve_until(expires_at)
    }

    pub(in crate::services::browser) fn fail(&self, category: CefUnavailableCategory) {
        self.shared.fail(MacSupervisionFailure::External(category));
    }

    fn failure(&self) -> Option<CefUnavailableCategory> {
        failure_from_id(self.shared.failure.load(Ordering::Acquire))
    }
}
